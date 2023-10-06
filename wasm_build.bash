#!/bin/bash -e

set -o pipefail

# Build the WASM in `pkg`
wasm-pack build

# Create the 'www' directory if it does not exist.
[[ -d 'www' ]] || mkdir -p 'www'

# Create an NPM package from the template "wasm-app" in the `www` directory.
npm init wasm-app www

# Enter the `www` directory,...
pushd "www" >> /dev/null || exit 1

# ...and install the dependencies...
npm install

# ...And run the server.
npm run start > /dev/null &
server_pid=$!

echo 'Server at: localhost:8080'

kill "${server_pid}"

# Exit when you're done.
popd >> /dev/null || exit 1
