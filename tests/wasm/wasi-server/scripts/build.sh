#!/usr/bin/env bash

SCRIPTPATH="$( cd "$(dirname "$0")" ; pwd -P )"
cd $SCRIPTPATH
cd ..

rm wasi_server.wasm
export RUSTFLAGS="--cfg tokio_unstable"
export FD_COUNT="3"
cargo build --target wasm32-wasi --release
cp target/wasm32-wasi/release/wasi-server.wasm wasi-server.wasm
# wasmtime run ./wasi-server.wasm -W tcp-listen=127.0.0.1:8080
# --env FD_COUNT=3