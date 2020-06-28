#!/bin/bash

# Run from repo's root folder (relative to Gradle build file. TODO do this in Gradle)
# cd ../..

if [ -d "android/core/gradle" ]; then
    echo "Starting..."
else
    echo "Error: this script must be run in project root"
    exit 5
fi

root=$(pwd)

echo "Running make_rust.sh from $root"

# Building  ###########################################################

# See https://github.com/Co-Epi/app-backend-rust/wiki/Building-library-for-Android

if test -z "$PATH_TO_ANDROID_REPO"; then
    echo "Environment variable PATH_TO_ANDROID_REPO not set. Please set the variable using: 'export PATH_TO_ANDROID_REPO=<your_path_here>'"
    exit 6
else
    echo "\$PATH_TO_ANDROID_REPO is $PATH_TO_ANDROID_REPO"
    if [ -d "$PATH_TO_ANDROID_REPO" ]; then
        echo "Will copy files to ${PATH_TO_ANDROID_REPO}..."
    else
        echo "Error: ${PATH_TO_ANDROID_REPO} not found. Make sure that you have checked out the iOS project and that the path specified in PATH_TO_ANDROID_REPO environment variable is correct"
        exit 7
    fi
fi

cd android/core || exit 1

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

cd ../.. || exit 2

# Linking ###########################################################

#PATH_TO_ANDROID_LIBRARY="./android/core"
PATH_TO_ANDROID_MAIN=$root/android/core/core/src/main

echo "PATH_TO_ANDROID_MAIN is $PATH_TO_ANDROID_MAIN"

rm -fr $PATH_TO_ANDROID_MAIN/jniLibs
mkdir $PATH_TO_ANDROID_MAIN/jniLibs
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/armeabi
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/x86_64
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/x86

if [[ $* == *--release* ]]; then
    # release
    cp $root/target/aarch64-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a/libcoepi_core.so
    cp $root/target/x86_64-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86_64/libcoepi_core.so
    cp $root/target/armv7-linux-androideabi/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/armeabi/libcoepi_core.so
    cp $root/target/i686-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86/libcoepi_core.so
else
    # debug
    echo "Copying .so files to $PATH_TO_ANDROID_MAIN/jniLibs..."
    cp $root/target/aarch64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a/libcoepi_core.so
    cp $root/target/x86_64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86_64/libcoepi_core.so
    cp $root/target/armv7-linux-androideabi/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/armeabi/libcoepi_core.so
    cp $root/target/i686-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86/libcoepi_core.so

    echo "Copying .so files to  $PATH_TO_ANDROID_REPO/app/src/main/jniLibs..."
    cp $root/target/aarch64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_REPO/app/src/main/jniLibs/arm64-v8a/libcoepi_core.so
    cp $root/target/x86_64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_REPO/app/src/main/jniLibs/x86_64/libcoepi_core.so
    cp $root/target/armv7-linux-androideabi/debug/libcoepi_core.so $PATH_TO_ANDROID_REPO/app/src/main/jniLibs/armeabi/libcoepi_core.so
    cp $root/target/i686-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_REPO/app/src/main/jniLibs/x86/libcoepi_core.so
fi

echo "Done"