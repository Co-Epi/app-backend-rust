#!/bin/bash

# Requirements: 
# 1. https://github.com/getditto/rust-bitcode
# 2. libtool
# See https://github.com/Co-Epi/app-backend-rust/wiki/Building-library-for-iOS

RUSTFLAGS="-Z embed-bitcode" cargo +ios-arm64 build --target aarch64-apple-ios --release --lib
cargo build --target=x86_64-apple-ios --release

libtool -static -o ./target/CoEpiCore ./target/aarch64-apple-ios/release/libtcn_client.a ./target/x86_64-apple-ios/release/libtcn_client.a

# Overwrite library in iOS app (downloaded with Carthage) with local build.

PATH_TO_IOS_REPO="<insert path>"

PATH_TO_CARTHAGE_FRAMEWORK=$PATH_TO_IOS_REPO/Carthage/Build/iOS/CoEpiCore.framework
cp ./target/CoEpiCore $PATH_TO_CARTHAGE_FRAMEWORK/Versions/A/
cp ./src/ios/c_headers/coepicore.h $PATH_TO_CARTHAGE_FRAMEWORK/Versions/A/Headers/
