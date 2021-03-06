#!/bin/sh

## requires wasm-pack
## > curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

(
    cd ../..;
    mkdir -p ./avm/client/wasm || exit;
    wasm-pack build ./air-interpreter --no-typescript --release -d ../avm/client/wasm
)

cat << EOF > ./src/wasm.js
// auto-generated

module.exports = "$(base64 -w0 wasm/air_interpreter_client_bg.wasm)";
EOF
