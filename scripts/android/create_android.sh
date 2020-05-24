#!/bin/bash

# See https://github.com/Co-Epi/app-backend-rust/wiki/Building-library-for-Android

cargo build --target=aarch64-linux-android --release
cargo build --target=armv7-linux-androideabi --release
cargo build --target=i686-linux-android --release

