
FROM node:24-alpine

WORKDIR /opt/lilac/web

COPY ./frontend/package.json .

RUN npm install --legacy-peer-deps

RUN npm i -g serve

COPY ./frontend/ .

RUN npm run build

EXPOSE 8080

CMD [ "serve", "-s", "dist", "-l", "8080" ]