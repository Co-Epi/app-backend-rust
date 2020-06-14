#!/bin/bash

# See https://github.com/Co-Epi/app-backend-rust/wiki/Building-library-for-Android

# debug builds
cargo ndk --platform 29 --target x86_64-linux-android build
cargo ndk --platform 29 --target aarch64-linux-android build
cargo ndk --platform 29 --target armv7-linux-androideabi build

# release builds
# cargo ndk --platform 29 --target x86_64-linux-android build --release
# cargo ndk --platform 29 --target aarch64-linux-android build --release
# cargo ndk --platform 29 --target armv7-linux-androideabi build --release

