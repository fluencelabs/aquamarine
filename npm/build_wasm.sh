#!/bin/sh

## requires wasm-pack
## > curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

(
    cd ..
    wasm-pack build ./interpreter --no-typescript --release -d ./npm/wasm
)

cat << EOF > ./src/wasm.js
module.exports = "$(base64 -w0 wasm/aquamarine_client_bg.wasm)";
EOF