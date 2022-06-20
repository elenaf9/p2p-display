#!/bin/bash

SCRIPT_DIRECTORY="$(cd -- "$(dirname "$0")" > /dev/null 2>&1 ; pwd -P)"
cd $SCRIPT_DIRECTORY

if [[ -n "${CC}" ]] && [[ -n "${AR}" ]]; then
    cd ../display
    make lib CC=$CC AR=$AR
else
    cd ../display
    make lib
fi

cd ../management
cargo build --release --target arm-unknown-linux-gnueabihf $@
cp target/arm-unknown-linux-gnueabihf/release/management ./arm-binary
