#!/bin/bash

# Building  ###########################################################

# See https://github.com/Co-Epi/app-backend-rust/wiki/Building-library-for-Android

# debug builds
cargo ndk --platform 29 --target x86_64-linux-android build
cargo ndk --platform 29 --target aarch64-linux-android build
cargo ndk --platform 29 --target armv7-linux-androideabi build

# release builds
# cargo ndk --platform 29 --target x86_64-linux-android build --release
# cargo ndk --platform 29 --target aarch64-linux-android build --release
# cargo ndk --platform 29 --target armv7-linux-androideabi build --release

# Linking ###########################################################

PATH_TO_LIB_REPO="."
PATH_TO_ANDROID_REPO="<insert path>"

echo "Copying library files to Android app: ${PATH_TO_ANDROID_REPO}"

PATH_TO_ANDROID_MAIN=$PATH_TO_ANDROID_REPO/app/src/main

mkdir $PATH_TO_ANDROID_MAIN/jniLibs
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/arm64
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/armeabi
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/x86

# debug
ln -s $(pwd)/target/aarch64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64/libcoepi_core.so
ln -s $(pwd)/target/aarch64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a/libcoepi_core.so
ln -s $(pwd)/target/i686-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86/libcoepi_core.so
ln -s $(pwd)/target/armv7-linux-androideabi/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/armeabi/libcoepi_core.so

# release
# ln -s $(pwd)/target/aarch64-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64/libcoepi_core.so
# ln -s $(pwd)/target/aarch64-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a/libcoepi_core.so
# ln -s $(pwd)/target/i686-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86/libcoepi_core.so
# ln -s $(pwd)/target/armv7-linux-androideabi/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/armeabi/libcoepi_core.so
