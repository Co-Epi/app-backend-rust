#!/bin/bash

if test -z "$PATH_TO_ANDROID_REPO"; then
    echo "Environment variable PATH_TO_ANDROID_REPO not set. Please set the variable using: 'export PATH_TO_ANDROID_REPO=<your_path_here>'"
    exit 1
else
    echo "\$PATH_TO_ANDROID_REPO is $PATH_TO_ANDROID_REPO"
    if [ -d "$PATH_TO_ANDROID_REPO" ]; then
        echo "Will copy files to ${PATH_TO_ANDROID_REPO}..."
    else
        echo "Error: ${PATH_TO_ANDROID_REPO} not found. Make sure that you have checked out the Android project and that the path specified in PATH_TO_ANDROID_REPO environment variable is correct"
        exit 1
    fi
fi

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

echo "Copying library files to Android app: ${PATH_TO_ANDROID_REPO}"

PATH_TO_ANDROID_MAIN=$PATH_TO_ANDROID_REPO/app/src/main

mkdir $PATH_TO_ANDROID_MAIN/jniLibs
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/arm64
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/armeabi
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/x86_64

# debug
ln -s $(pwd)/target/aarch64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64/libcoepi_core.so
ln -s $(pwd)/target/aarch64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a/libcoepi_core.so
ln -s $(pwd)/target/x86_64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86_64/libcoepi_core.so
ln -s $(pwd)/target/armv7-linux-androideabi/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/armeabi/libcoepi_core.so

# release
# ln -s $(pwd)/target/aarch64-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64/libcoepi_core.so
# ln -s $(pwd)/target/aarch64-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a/libcoepi_core.so
# ln -s $(pwd)/target/x86_64-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86_64/libcoepi_core.so
# ln -s $(pwd)/target/armv7-linux-androideabi/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/armeabi/libcoepi_core.so
