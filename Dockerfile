FROM node:lts-stretch

WORKDIR /fcheck

RUN apt-get update &&\
    # Diff tool, Kafka tool
    apt-get install -y kafkacat wdiff 

COPY package.json /fcheck/

RUN npm install &&\
    mkdir /fcheck/config &&\
    mkdir /fcheck/data

COPY index.js /fcheck/

# CMD ["bash"]
ENTRYPOINT ["node", "index.js"]
