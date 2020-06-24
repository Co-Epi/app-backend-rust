#!/bin/bash

# Run from repo's root folder (relative to Gradle build file. TODO do this in Gradle)
cd ../..
echo "Running make_rust.sh from $(pwd)"

# Building  ###########################################################

# See https://github.com/Co-Epi/app-backend-rust/wiki/Building-library-for-Android

if [[ $* == *--release* ]]; then
    # release builds
    cargo ndk --platform 29 --target x86_64-linux-android build --release
    cargo ndk --platform 29 --target aarch64-linux-android build --release
    cargo ndk --platform 29 --target armv7-linux-androideabi build --release
    cargo ndk --platform 29 --target i686-linux-android build --release
else
    # debug builds
    cargo ndk --platform 29 --target x86_64-linux-android build
    cargo ndk --platform 29 --target aarch64-linux-android build
    cargo ndk --platform 29 --target armv7-linux-androideabi build
    cargo ndk --platform 29 --target i686-linux-android build
fi


# Linking ###########################################################

echo "Copying .so files to Android project"

PATH_TO_ANDROID_LIBRARY="./android/core"
PATH_TO_ANDROID_MAIN=$PATH_TO_ANDROID_LIBRARY/app/src/main

rm -fr $PATH_TO_ANDROID_MAIN/jniLibs
mkdir $PATH_TO_ANDROID_MAIN/jniLibs
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/arm64
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/armeabi
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/x86_64
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/x86

if [[ $* == *--release* ]]; then
    # release
    cp $(pwd)/target/aarch64-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64/libcoepi_core.so
    cp $(pwd)/target/aarch64-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a/libcoepi_core.so
    cp $(pwd)/target/x86_64-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86_64/libcoepi_core.so
    cp $(pwd)/target/armv7-linux-androideabi/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/armeabi/libcoepi_core.so
    cp $(pwd)/target/i686-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86/libcoepi_core.so
else
    # debug
    cp $(pwd)/target/aarch64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64/libcoepi_core.so
    cp $(pwd)/target/aarch64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a/libcoepi_core.so
    cp $(pwd)/target/x86_64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86_64/libcoepi_core.so
    cp $(pwd)/target/armv7-linux-androideabi/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/armeabi/libcoepi_core.so
    cp $(pwd)/target/i686-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86/libcoepi_core.so
fi
