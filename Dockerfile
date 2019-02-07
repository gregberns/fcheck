FROM node:lts-stretch

WORKDIR /fcheck

RUN apt-get update &&\
    apt-get install -y kafkacat wdiff netcat

COPY package.json /fcheck/

RUN npm install &&\
    mkdir /fcheck/config &&\
    mkdir /fcheck/data &&\
    mkdir /fcheck/output

COPY index.js /fcheck/

ENTRYPOINT ["node", "index.js"]
