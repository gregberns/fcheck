#!/usr/bin/env node

const program = require('commander');
const toml = require('toml');
const concat = require('concat-stream');
const fs = require('fs');
const kafka = require('kafka-node');
const child_process = require('child_process');

console.log('fcheck starting')

program
  .version('0.1.0')
  .option('-c, --config-file [file]', 'Configuration file containing tests to be run', './config/config.toml')
  .option('-r, --report-file [file]', 'File with test configuration', './data/report.json')
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
      // console.log(data)
      return parseToml(data)
    })
    .then(config => {
      // console.log(config)
      return runTests(config.test)
    })
    .then(results => {
      console.log(results)
      writeFile(program.reportFile, JSON.stringify(results,  undefined, 2))
      console.log(`Report file written to: ${program.reportFile}`)
    })
    .catch(error => {
      console.error(error)
    })
}

const parseToml = data => {
  return new Promise((resolve, reject) => {
    try {
      resolve(toml.parse(data))
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
      if(err) return reject(new Error(`Failed to write file: ${filepath}`, e))
      resolve()
    });
  })
}

const runTests = async tests => {
  let promiseArray = 
    tests.map(test  => {
      return runTest(test.name, test)
    })

  return Promise.all(promiseArray)
}

const runTest = async (testName, config) => {
  if (config.disabled) 
    return { 
      testName,
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

const runProcess = command => {
  return new Promise((resolve, reject) => {
    try {
      let buffer = child_process.execSync(command)
      resolve(buffer)
    } catch (e) {
      reject(e)
    }
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
