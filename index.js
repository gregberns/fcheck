#!/usr/bin/env node

const program = require('commander');
const toml = require('toml');
const concat = require('concat-stream');
const fs = require('fs');
const kafka = require('kafka-node');
const child_process = require('child_process');
const util = require('util');
var StringDecoder = require('string_decoder').StringDecoder;

console.log('fcheck starting')

program
  .version('0.1.0')
  .option('-c, --config-file [file]', 'Configuration file containing tests to be run', './config/config.toml')
  .option('-r, --report-file [file]', 'File with test configuration', './data/report.json')
  .option('-v, --verbose-errors', 'Verbose error logging')
  .parse(process.argv);

if (!program.configFile) {
  console.error('No test file provided.')
  return
}
let configFile = program.configFile
console.log('ConfigFile location: ' + configFile);

function run(configFileLocation) {
  readFile(configFileLocation)
    .then(data => {
      if (configFile.endsWith('.toml')) {
        return parseToml(data)
      }
      if (configFile.endsWith('.dhall')) {
        return parseDhall(data)
      }
      throw new Exception(`Config file did not end in a valid format: ${configFile}. Supported formats: .toml, .dhall`)
    })
    .then(config => {
      console.log('config', JSON.stringify(config,  undefined, 2))
      return runProcesses(config)
    })
    .then(async results => {
      console.log(JSON.stringify(results,  undefined, 2))
      
      await writeFile(program.reportFile, JSON.stringify(results,  undefined, 2))

      console.log(`Report file written to: ${program.reportFile}`)
      
      if (results.result === 'success') {
        process.exit(0)  
      } else {
        console.error(`Process failed. Exit code 1`)
        process.exit(1)
      }
    })
    .catch(error => {
      console.error(error)
      process.exit(2)
    })
}

const runProcesses = async config => {
  let setup = undefined
  if (config.setup && config.setup[0]) {
    setup = await runTest("Setup", config.setup[0])
    if (setup.result === 'failure') {
      return {
        result: 'failure',
        setup,
      }
    }
  }

  let tests = await runTests(config.test)

  let teardown = undefined;
  if (config.teardown && config.teardown[0]) {
    // dont stop the tear down from running if one of the tests fail to run
    teardown = await runTest("Teardown", config.teardown[0])

  }
  
  let failures = tests.filter(test => test.result === 'failure').length
  let result = failures === 0 ? 'success' : 'failure'

  return {
    result,
    setup,
    tests,
    teardown,
  }
}

const runTests = async tests => {
  if (tests == null) return new Promise((resolve) => {resolve()})
  let promiseArray = 
    tests.map((test, index)  => {
        let testName = test.name && `Process/Test #${index+1}`
        return runTest(test.name, test)
      })

  return Promise.all(promiseArray)
}

const runTest = async (name, config) => {
  if (config.disabled) 
    return { 
      name,
      result: 'disabled' 
    }
  
  let version = config.version || 2
  if (version === 1) {
    return await runTestV1(name, config)
  } else if (version === 2) {
    // console.log("------------")
    // console.log(JSON.stringify(config,  undefined, 2))
    return await runTestV2(name, config)
  } else {
    throw new Error(`Version in config not supported: ${version}. Supported: 2`)
  }
}

const runTestV2 = async (name, test) => {
  if (test.disabled) {
    return { 
      name,
      result: 'disabled' 
    }
  }

  console.log(`runTestV2: ${name}. Parallel: ${test.parallel?'true':'false'}`)

  if (test.parallel) {
    let runnable = test.command
      .map(async (commandObj, index) => {
        let commandName = commandObj.name || `Command # ${index+1}`
        console.log(`Start RunProcess. Command Name: ${commandName}`)
        
        return runProcess(commandObj.command, commandObj.timeout)
          .then(result => {
            console.log(`End RunProcess. Command Name: ${commandName}`)
            let formattedResult = (result instanceof Buffer) ? result.toString() : result
            return {
              commandName: commandName,
              commandResult: 'success',
              commandCommand: commandObj.command,
              commandOutput: formattedResult,
            }
          })
          .catch(error => {
            console.log(`End RunProcess. Command Name: ${commandName}`)
            throw {
              commandName: commandObj.name || `Command # ${index+1}`,
              commandResult: 'failure',
              commandCommand: commandObj.command,
              commandOutput: error,
            }
          })
      })
      
    return Promise.all(runnable)
      .then(results => {
        // let failures = results.filter(test => test.result === 'failure').length
        // let result = failures === 0 ? 'success' : 'failure'
        return {
          name,
          result: 'success',
          results
        }
      })
      .catch(results => {
        return {
          name,
          result: 'failure',
          results
        }
      })
  } else {

    let results = []
    for (let index = 0; index < test.command.length; ++index) {
      let commandObj = test.command[index]
      let commandName = commandObj.name || `Command # ${index+1}`
      console.log(`Start RunProcess. Command Name: ${commandName}`)
      
      await runProcess(commandObj.command, commandObj.timeout)
        .then(result => {
          console.log(`End RunProcess. Command Name: ${commandName}`)
          let formattedResult = (result instanceof Buffer) ? result.toString() : result
          return {
            commandName: commandName,
            commandResult: 'success',
            commandCommand: commandObj.command,
            commandOutput: formattedResult,
          }
        })
        .catch(error => {
          console.log(`End RunProcess. Command Name: ${commandName}`)
          return {
            commandName: commandObj.name || `Command # ${index+1}`,
            commandResult: 'failure',
            commandCommand: commandObj.command,
            commandOutput: error,
          }
        })
        .then(result => results.push(result))
    }

    console.log('res', results)

    let failures = results.filter(test => test.commandResult === 'failure').length

    console.log('res', results)

    return {
      name,
      result: failures > 0 ? 'failure' : 'success',
      results
    }
    
    // return Promise.all(runnable)
    //   .then(results => {
    //     // let failures = results.filter(test => test.result === 'failure').length
    //     // let result = failures === 0 ? 'success' : 'failure'
    //     return {
    //       name,
    //       result: 'success',
    //       results
    //     }
    //   })
    //   .catch(results => {
    //     return {
    //       name,
    //       result: 'failure',
    //       results
    //     }
    //   })
  }
}

const runTestV1 = async (name, config) => {
  if (config.disabled) 
    return { 
      name,
      result: 'disabled' 
    }
  
  try {
    let inputResult = 
      await runCommand(config.input, config.timeout)

    let outputResult =
      await runCommand(config.output, config.timeout)
  
    let validateResult = 
      await runCommand(config.validate, config.timeout)

    return {
      testName,
      result: 'success',
      results: [
        inputResult,
        outputResult,
        validateResult
      ]
    }
  } catch (e) {
    return {
      testName,
      result: 'failure',
      error: e
    }
  }
}

const runCommand = async (config, timeout) => {
  var returnValue = null
  if (config.type === 'command') {
    returnValue = await runProcess(config.command, timeout)
  }
  if (config.type === "file") {
    if (config.operation === "copy") {
      await copyFile(config.fileFrom, config.fileTo)
      return {
        type: config.type
      }
    }
  }
  if (config.type === 'kafka') {
    let host = config.host
    let topic = config.topic
    if (config.operation === 'write') {
      await writeKafkaMessages(host, topic, config.messages)
    }
    if (config.operation === 'read') {
      let messages = await readKafkaMessage(host, topic)
      returnType = messages
    }
  }
  if (config.delay) {
    await sleep(config.delay)
  }

  return {
    type: config.type,
    returnValue
  }
}

const runProcess = (command, timeout) => {
  return new Promise((resolve, reject) => {
    try {
      let buffer = child_process.execSync(command, {timeout})
      resolve(buffer)
    } catch (e) {
      var err = parseProcessError(e)
      reject(err)
    }
  })
}

const parseProcessError = e => {
  try {
    return { 
      parsedError: {
        code: e.code,
        //error: e.Error,
        status: e.status,
        output: e.output.map(i => i && i.toString()),
        stdout: e.stdout.toString(),
        stderr: e.stderr.toString()
      },
      rawError: program.verboseErrors ? e : 'Not displayed. Include `-v` to include verbose errors.',
    }
  } catch (err) {
    console.error("-------------")
    console.error('Failed to process error 1', e)
    console.error("-------------")
    console.error('Failed to process error 2', err)
    console.error("-------------")
    return { 
      rawError: e,
      parseProcessError: err
    }
  }
}

// { testName: 'ping zookeeper',
// fcheck_1     |     result: 'failure',
// fcheck_1     |     error:
// fcheck_1     |      { Error: Command failed: nc -z zookeeper 2181 > /dev/null 2>&1
// fcheck_1     |          at checkExecSyncError (child_process.js:616:11)
// fcheck_1     |          at Object.execSync (child_process.js:653:13)
// fcheck_1     |          at Promise (/fcheck/index.js:157:34)
// fcheck_1     |          at new Promise (<anonymous>)
// fcheck_1     |          at runProcess (/fcheck/index.js:155:10)
// fcheck_1     |          at runCommand (/fcheck/index.js:123:25)
// fcheck_1     |          at runTest (/fcheck/index.js:94:13)
// fcheck_1     |          at tests.map.test (/fcheck/index.js:79:14)
// fcheck_1     |          at Array.map (<anonymous>)
// fcheck_1     |          at runTests (/fcheck/index.js:78:11)
// fcheck_1     |        status: 127,
// fcheck_1     |        signal: null,
// fcheck_1     |        output: [Array],
// fcheck_1     |        pid: 17,
// fcheck_1     |        stdout: <Buffer >,
// fcheck_1     |        stderr: <Buffer > } } ]


const parseToml = data => {
  return new Promise((resolve, reject) => {
    try {
      resolve(toml.parse(data))
    } catch (e) {
      reject(e)
    }
  })
}

const parseDhall = data => {
  return new Promise((resolve, reject) => {
    try {
      // console.log('parseDhall', data)
      // let buffer = child_process.execSync(`dhall-to-json <<< '${data}'`, { options: {input: data}})
      let buffer = child_process.execSync(`dhall-to-json`, { input: data, stdio: 'pipe', shell: '/bin/bash'})
      let obj = JSON.parse(buffer.toString())
      // console.log('parseDhall res', obj)
      resolve(obj)
    } catch (e) {
      reject(e)
    }
  })
}

const readFile = (filepath) => {
  return new Promise((resolve, reject) => {
    fs.createReadStream(filepath, 'utf8')
      .pipe(concat(function(data) {
        resolve(data);
      }));
  })
}

const writeFile = (filepath, contents) => {
  return new Promise((resolve, reject) => {
    fs.writeFile(filepath, contents, function(err) {
      if (err) return reject(new Error(`Failed to write file: ${filepath}`, err))
      resolve()
    });
  })
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

const copyFile = (source, destination) => {
  return new Promise((resolve, reject) => {
    fs.copyFile(source, destination, (err) => {
      if (err) reject(err)
      console.log(`${source} was copied to ${destination}`);
      resolve()
    });
  })
}

const getKafkaProducer = (host, topic) => {
  const client = new kafka.KafkaClient({
    kafkaHost: host
  });
  let Producer = kafka.Producer
  let producer = new Producer(
      client,
      [
          { topic: topic }
      ],
      {
          autoCommit: false
      }
  );
  return producer
}

const getKafkaConsumer = (host, topic) => {
  const client = new kafka.KafkaClient({
    kafkaHost: host
  });
  let Consumer = kafka.Consumer
  let consumer = new Consumer(
      client,
      [
          { topic: topic }
      ],
      {
          autoCommit: false
      }
  );
  return consumer
}

const writeKafkaMessages = async (host, topic, messages) => {
  return new Promise((resolve, reject) => {
    try {
      getKafkaProducer(host)
        .send({
          topic: topic,
          messages: messages, // multi messages should be a array, single message can be just a string or a KeyedMessage instance
          // key: 'theKey', // string or buffer, only needed when using keyed partitioner
          // partition: 0, // default 0
          // attributes: 2, // default: 0
          timestamp: Date.now() // <-- defaults to Date.now() (only available with kafka v0.10+)
      })
      resolve()
    } catch (e) {
      reject(e)
    }
  })
}

const readKafkaMessage = async (host, topic) => {
  return new Promise((resolve, reject) => {
    try {
      getKafkaConsumer(host, topic).on('message', function (message) {
        resolve(message)
      });
    } catch (e) {
      reject(e)
    }
  })
}

run(configFile)
