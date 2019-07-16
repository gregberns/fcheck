FROM node:lts-stretch

WORKDIR /fcheck

RUN apt-get update &&\
    apt-get install -y wdiff netcat

COPY ./dhall/dhall-to-json /usr/local/bin
COPY ./dhall/dhall-to-yaml /usr/local/bin
COPY ./wait-for-it.sh /

COPY package.json /fcheck/
RUN npm install &&\
    mkdir /fcheck/config &&\
    mkdir /fcheck/data &&\
    mkdir /fcheck/output

COPY index.js /fcheck/

ENTRYPOINT ["node", "index.js"]
