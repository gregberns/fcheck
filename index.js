#!/usr/bin/env node

const program = require('commander');
const toml = require('toml');
const concat = require('concat-stream');
const fs = require('fs');
const kafka = require('kafka-node');
const child_process = require('child_process');

console.log(process.argv)

program
  .version('0.1.0')
  .option('-t, --test-file [file]', 'File with test configuration')
  .parse(process.argv);


if (!program.testFile) {
  console.error('No test file provided.')
  return
}
let configFile = program.testFile
console.log('TestFile location: ' + configFile);

function run(configFileLocation) {
  readFile(configFileLocation)
    .then(data => {
      console.log(data)
      return parseToml(data)
    })
    .then(config => {
      console.log(config)
      return runTests(config.tests)
    })
    .then(results => {
      console.log(results)
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
      // reject(new Error('Failed to parse config file', e))
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

const runTests = async tests => {
  let results = {}
  
  for (let testName in tests) {
    let value = tests[testName]

    results[testName] = await runTest(testName, value)    
  }
  return results
}

const runTest = async (testName, config) => {
  if (config.disabled) 
    return { result: 'disabled' }
  
  try {
    await runCommand(config.input, config.timeout)

    await runCommand(config.output, config.timeout)
  
    await runCommand(config.validate, config.timeout)

    return {
      result: 'success'
    }
  } catch (e) {
    return {
      result: 'failure',
      error: e
    }
  }
}

const runCommand = async (config, timeout) => {
  var returnValue = null
  if (config.type === 'command') {
    await runProcess(config.command, timeout)
    
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

run(configFile)
