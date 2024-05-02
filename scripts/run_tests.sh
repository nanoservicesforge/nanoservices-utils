#!/usr/bin/env bash

SCRIPTPATH="$( cd "$(dirname "$0")" ; pwd -P )"
cd $SCRIPTPATH
cd ..

cargo test --lib --features networking
cargo test --lib --features wasm-messaging
cargo test --lib --features tcp-messaging
