# fcheck - Testing Framework

What would a language agnostic, integration test framework look like?

Maybe something like Postman(an Http testing tool) or Newman(its command line executor), but a tool that would:

* Run general setup commands
* Execute general commands to test a feature
  * Run a command to start a process
  * Check for updated values
  * Validate the results
* Run teardown commands

To be as flexible as possible and not re-invent the wheel, we can leverage local OS commands. So on Linux, we can run bash/sh commands: to move files, run Kafka commands, make HTTP requests with curl, or any other commands that the OS supports or that are installed.

If the OS or Docker image doesn't have the command, then just install the command prior to running the tests!

**ATTENTION** This is a v0.* version! Expect bugs and issues all around. Submitting pull requests and issues is highly encouraged!

## Why - The Problem To Be Solved

Testing microservices can be hard, especially within the context of a distributed system. We could write a set of scripts to execute these tests, but writing and debugging them can be difficult and maintainting and extending them can be unbearable.

This project attempts to simplify this problem.

What types of problems does this solve:

### Example Service: Kafka Producer

Test a service that is a Kafka producer: Service reads a file, then puts a message on the queue.

What would the test look like?

* Setup: ensure Kafka has a the appropriate topic
* Test:
  * Copy file where the service can see it
  * Sleep for 250ms
  * Read last message from Kafka and save to output location
  * Diff the output file with an expected result

## Getting Started

To run an example config file:

```bash
docker run -v ${PWD}/config/:/fcheck/config/ -v ${PWD}/data/:/fcheck/data/ fcheck -c ./examples/config.toml -r ./data/report.json
```

## Commands Available

The fcheck Docker image has several tools built in:

* Commands available in: `node:lts-stretch`, Debian `stretch`, etc
* `kafkacat`
* `wdiff`
* `netcat` or `nc`

## Docker Support

Docker support is first-class, but to solve your particular problem, you may need to extend the base fcheck image with the binary's you need to use.

Dockerfile

```Dockerfile
FROM fcheck

WORKDIR /fcheck

# by default pass the config file location (`c`) and output location (`-r`)
CMD -c /fcheck/config/config.toml -r /fcheck/output/report.json

# if you want to override the default behavior and run a 
# command before the process begins, run like this:
ENTRYPOINT []
CMD sleep 15s &&\
    node index.js -c /fcheck/config/config.toml -r /fcheck/output/report.json
```

docker-compose.yml:

```yml
version: '3.7'

services:
  fcheck:
    build: 
      context: .
      dockerfile: ./fcheck/Dockerfile
    volumes:
      - ./fcheck/config:/fcheck/config
      - ./fcheck/data:/fcheck/data
      - ./fcheck/output:/fcheck/output
```

## Configuration

Currently, fcheck supports TOML files as the default configuration method.

In future implementations, we'd like to support the [Dhall configuration language](https://dhall-lang.org/). This will help reduce duplicate declaration of paths, files, commands, etc, with an aim to improve maintainability and robustness of our testing frameworks.

### Dhall Support

Dhall is a configuration language that allows you to remove **all** the duplication you generally see when dealing with configuration files.

For example, filenames, paths, server names, and environment values can all be declared once, so instead of changing them in multiple places, like in a normal script, you can change them in a single place. Simple string interpolation is used to use the variable name.

See examples of usage below.

If you already have a `.dhall` config file, just pass the filename, including the `.dhall` extension. To test a Dhall config file:

```bash
dhall-to-json < ./examples/config.dhall
```

#### Running Dhall Locally

To run fcheck locally on Mac or Windows (outside the Docker image), you will want to read through the [Dhall Getting Started Guide](https://github.com/dhall-lang/dhall-lang/wiki/Getting-started%3A-Generate-JSON-or-YAML).

Install on Mac

```
brew install dhall-json
```

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

## Contributing

To run the project on the local system:

```bash
node index.js -c ./examples/configv2.toml
```

To build the project in Docker:

```bash
docker build -t fcheck .
```

To run in docker:

```bash
docker run -v ${PWD}/config/:/fcheck/config/ -v ${PWD}/data/:/fcheck/data/ fcheck -c ./examples/config.toml -r ./data/report.json
```

### Rust

```bash
cargo build
cargo run -- -c ./config/config-v3.toml
```
