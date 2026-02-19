# install (yarn still needed for npm deps with GitHub references)
FROM node:22-slim as install
WORKDIR /usr/src/app
COPY package.json yarn.lock ./
RUN yarn install

# build (Deno as runtime)
FROM denoland/deno:2 as build
WORKDIR /usr/src/app
COPY --from=install /usr/src/app/node_modules ./node_modules
COPY . .
RUN deno task build
