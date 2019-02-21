FROM node:lts-stretch

WORKDIR /fcheck

RUN apt-get update &&\
    apt-get install -y kafkacat wdiff netcat &&\
    npm install -g newman

COPY package.json /fcheck/
COPY ./dhall/dhall-to-json /usr/local/bin
COPY ./dhall/dhall-to-yaml /usr/local/bin

RUN npm install &&\
    mkdir /fcheck/config &&\
    mkdir /fcheck/data &&\
    mkdir /fcheck/output

COPY index.js /fcheck/

ENTRYPOINT ["node", "index.js"]
