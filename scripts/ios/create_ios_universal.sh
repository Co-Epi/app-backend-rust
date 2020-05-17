#!/bin/bash

# Requirements: 
# 1. https://github.com/getditto/rust-bitcode
# 2. libtool
# See https://github.com/Co-Epi/app-backend-rust/wiki/Building-library-for-iOS

RUSTFLAGS="-Z embed-bitcode" cargo +ios-arm64 build --target aarch64-apple-ios --release --lib
cargo build --target=x86_64-apple-ios --release

libtool -static -o ./target/libtcn_client.a ./target/aarch64-apple-ios/release/libtcn_client.a ./target/x86_64-apple-ios/release/libtcn_client.a

# Copy lib into iOS app
# mv ./target/libtcn_client.a <path to iOS app's root dir>

