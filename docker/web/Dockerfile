
FROM node:24-alpine

ARG VITE_LILAC_API_ENDPOINT="http://localhost:8081"
ENV VITE_LILAC_API_ENDPOINT=$VITE_LILAC_API_ENDPOINT

WORKDIR /opt/lilac/web

COPY ./frontend/package.json .

RUN npm install --legacy-peer-deps

RUN npm i -g serve

COPY ./frontend/ .

RUN npm run build

EXPOSE 8080

CMD [ "serve", "-s", "dist", "-l", "8080" ]