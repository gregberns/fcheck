# fcheck - Testing Framework

What would a language agnostic, integration test framework look like?

Maybe something like Postman / Newman(command line executor), but a tool that would execute general commands to kick off a test, check for updated values, and validate the results.

**ATTENTION** This is a v0.* version! Expect bugs and issues all around. Submitting pull requests and issues is highly encouraged!

## Why

### Problem to be solved

* Make HTTP request (which puts message on queue), reads Kafka queue, validates message contents
* Copy file to location (which is picked up by esrvice), make an HTTP request, and validate response

### Framework Requirements

* Supports various input and output sources
* Flexible validation mechanism (JS functions)
* Describe tests with declaritive syntax

## Getting Started

To run an example config file:

```bash
docker run -v ${PWD}/config/:/fcheck/config/ -v ${PWD}/data/:/fcheck/data/ fcheck -c ./examples/config.toml -r ./data/report.json
```

## Commands Available

* Commands available in: `node:lts-stretch`, Debian `stretch`, etc
* `kafkacat`
* `wdiff`
* `netcat` or `nc`

## Contributing

To run the project on the local system:

```bash
node index.js -c ./examples/config.toml
```

To build the project in Docker:

```bash
docker build -t fcheck .
```

To run in docker:

```
docker run -v ${PWD}/config/:/fcheck/config/ -v ${PWD}/data/:/fcheck/data/ fcheck -c ./examples/config.toml -r ./data/report.json
```

## Scenarios

* Producer with File Drop
  * Drop file in location
  * Wait
  * Read queue
  * Validate messages on queue
* Producer with HTTP endpoint
  * Send HTTP message
  * Wait
  * Read queue
  * Validate messages on queue
* Consumer check File
  * Push messages to queue
  * Wait
  * Read file location
  * Validate file contents

## Example

Configuration file has a setup, two tests, and a teardown. The first test will fail with an error.

Below is the output of running this config.

The failure occurs in `test1` because `rm ./data/cats.txt` does not include `-f`, which prevents failure if the file doesn't exist.

```toml
[[setup]]
[[setup.command]]
command = "rm -f ./data/cats.txt && rm -f ./data/dogs.txt"

[[test]]
name = "test1"
description = "Write two files and check they are the same"
disabled = false
timeout = 5000
[[test.command]]
command = "rm -f ./data/cats.txt && rm ./data/cats.txt && rm -f ./data/dogs.txt"
[[test.command]]
name = "Create Dogs file"
command = "echo \"Dogs\" > ./data/dogs.txt"
[[test.command]]
name = "Create Cats file"
command = "echo \"Dogs\" > ./data/cats.txt"
[[test.command]]
name = "diff"
command = "diff ./data/dogs.txt ./data/cats.txt"



[[test]]
name = "test2"
description = "Write two files and check they are the same"
disabled = false
timeout = 5000
[[test.command]]
command = "rm -f ./data/cats.txt && rm -f ./data/dogs.txt"
[[test.command]]
name = "Create Dogs file"
command = "echo \"Dogs\" > ./data/dogs.txt"
[[test.command]]
name = "Create Cats file"
command = "echo \"Dogs\" > ./data/cats.txt"
[[test.command]]
name = "diff"
command = "diff ./data/dogs.txt ./data/cats.txt"

[[teardown]]
[[teardown.command]]
command = "rm -f ./data/cats.txt && rm -f ./data/dogs.txt"
```

The output report shows:
* The setup was successful
* The first test failed
* The second test was successful
* The teardown was successful

```json
{
  "result": "failure",
  "setup": {
    "name": "Setup",
    "result": "success",
    "results": [
      {
        "commandName": "Command # 1",
        "commandResult": "success",
        "commandCommand": "rm -f ./data/cats.txt && rm -f ./data/dogs.txt",
        "commandOutput": ""
      }
    ]
  },
  "tests": [
    {
      "name": "test1",
      "result": "failure",
      "results": {
        "commandName": "Command # 1",
        "commandResult": "failure",
        "commandCommand": "rm -f ./data/cats.txt && rm ./data/cats.txt && rm -f ./data/dogs.txt",
        "commandOutput": {
          "parsedError": {
            "status": 1,
            "output": [
              null,
              "",
              "rm: ./data/cats.txt: No such file or directory\n"
            ],
            "stdout": "",
            "stderr": "rm: ./data/cats.txt: No such file or directory\n"
          },
          "rawError": "..."
        }
      }
    },
    {
      "name": "test2",
      "result": "success",
      "results": [
        {
          "commandName": "Command # 1",
          "commandResult": "success",
          "commandCommand": "rm -f ./data/cats.txt && rm -f ./data/dogs.txt",
          "commandOutput": ""
        },
        {
          "commandName": "Create Dogs file",
          "commandResult": "success",
          "commandCommand": "echo \"Dogs\" > ./data/dogs.txt",
          "commandOutput": ""
        },
        {
          "commandName": "Create Cats file",
          "commandResult": "success",
          "commandCommand": "echo \"Dogs\" > ./data/cats.txt",
          "commandOutput": ""
        },
        {
          "commandName": "diff",
          "commandResult": "success",
          "commandCommand": "diff ./data/dogs.txt ./data/cats.txt",
          "commandOutput": ""
        }
      ]
    }
  ],
  "teardown": {
    "name": "Teardown",
    "result": "success",
    "results": [
      {
        "commandName": "Command # 1",
        "commandResult": "success",
        "commandCommand": "rm -f ./data/cats.txt && rm -f ./data/dogs.txt",
        "commandOutput": ""
      }
    ]
  }
}
```
