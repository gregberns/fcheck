# fcheck - Testing Framework

What would a language agnostic, integration test framework look like?

## Why

### Problem to be solved

* Make HTTP request (which puts message on queue), reads Kafka queue, validates message contents
* Copy file to location (which is picked up by esrvice), make an HTTP request, and validate response

### Framework Requirements

* Supports various input and output sources
* Flexible validation mechanism (JS functions)
* Describe tests with declaritive syntax

## Getting Started

```bash
docker run -v ./config/:/fcheck/config/ ./data/:/fcheck/data/  gregberns/fcheck -c ./config/config.toml -r ./data/report.json


docker run -v ${PWD}/config/:/fcheck/config/ -v ${PWD}/data/:/fcheck/data/ fcheck -c ./config/config.toml -r ./data/report.json


```


## Contributing

To run the project on the local system:

```bash
node index.js -c ./config/config.toml
```

To build the project in Docker:

```bash
docker build -t fcheck .
```

docker run -v ${PWD}/config/:/fcheck/config/ -v ${PWD}/data/:/fcheck/data/ fcheck -c ./config/config.toml -r ./data/report.json




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




# compare using... literal, file,tool
# dump from database to csv - whole table
# 
# input output folder

# assert 'a' before and 'b' after

#emit 
see item is in the list - check for accoutn number
then emit message
then check that value has changed
only 
Then run newman...


pro


Account Event Producer
_ copy file to location
_ read messages off kafka - save to file
_ diff files

Account Filter Worker/Consumer
_ push messages onto kafka
_ newman run tests


use curl and not postman


test-data
  test1
    run.sh

    input.txt
    output.txt
    config.json

input - evt prod - copy file, start kafka reader

toml, array of commands to run
 - output check - agains a file

docker run testy -t test1 -v /test-dir:
pass params into compose?

bake in curl, kafka prod/consum


Account



Event 1 - Acct1
Acct1 - missing from store
Send "New Account" - Acct1 (whole payload)


Event 2 - Acc1
Acc1 - Exists in store
Diff
Send Diff


