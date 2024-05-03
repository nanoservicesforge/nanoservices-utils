#!/usr/bin/env bash

SCRIPTPATH="$( cd "$(dirname "$0")" ; pwd -P )"
cd $SCRIPTPATH
cd ..

rm wasi_server.wasm
export RUSTFLAGS="--cfg tokio_unstable"
export FD_COUNT="3"
cargo build --target wasm32-wasi --release
cp target/wasm32-wasi/release/wasi_server.wasm wasi_server.wasm
wasm2wat wasi_server.wasm > wasi_server.wat