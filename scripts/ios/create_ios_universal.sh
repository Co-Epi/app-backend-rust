#!/bin/bash

# Requirements: 
# 1. https://github.com/getditto/rust-bitcode
# 2. libtool
# See https://github.com/Co-Epi/app-backend-rust/wiki/Building-library-for-iOS

if test -z "$PATH_TO_IOS_REPO" 
then	
      echo "Environment variable PATH_TO_IOS_REPO not set. Please set the variable using: 'export PATH_TO_IOS_REPO=<your_path_here>'";
      exit 1
else
      echo "\$PATH_TO_IOS_REPO is $PATH_TO_IOS_REPO";
      if [ -d "$PATH_TO_IOS_REPO" ]; then
		  echo "Will copy files to ${PATH_TO_IOS_REPO}..."
		else
		  echo "Error: ${PATH_TO_IOS_REPO} not found. Make sure that you have checked out the iOS project and that the path specified in PATH_TO_IOS_REPO environment variable is correct"
		  exit 1
		fi
fi

RUSTFLAGS="-Z embed-bitcode" cargo +ios-arm64 build --target aarch64-apple-ios --release --lib
cargo build --target=x86_64-apple-ios --release

libtool -static -o ./target/CoEpiCore ./target/aarch64-apple-ios/release/libcoepi_core.a ./target/x86_64-apple-ios/release/libcoepi_core.a

# Overwrite library in iOS app (downloaded with Carthage) with local build.

#Set PATH_TO_IOS_REPO as environment variable. ie. 'export PATH_TO_IOS_REPO=...'
PATH_TO_CARTHAGE_FRAMEWORK=$PATH_TO_IOS_REPO/Carthage/Build/iOS/CoEpiCore.framework
cp ./target/CoEpiCore $PATH_TO_CARTHAGE_FRAMEWORK/Versions/A/
cp ./src/ios/c_headers/coepicore.h $PATH_TO_CARTHAGE_FRAMEWORK/Versions/A/Headers/
