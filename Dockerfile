# install + build (Deno for deps, Node for npx/rspack)
FROM denoland/deno:2 as build

# Install Node.js (needed for npx rspack)
RUN apt-get update && apt-get install -y curl && \
    curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y nodejs && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY package.json deno.json ./
RUN deno install

COPY . .
RUN deno task build
