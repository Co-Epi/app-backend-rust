#!/bin/bash

# Run from repo's root folder (relative to Gradle build file. TODO do this in Gradle)

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
        echo "Error: ${PATH_TO_ANDROID_REPO} not found. Make sure that you have checked out the corresponding Android project and that the path specified in PATH_TO_ANDROID_REPO environment variable is correct"
        exit 7
    fi
fi

# Order of target_triples has to match the order of arhitectures:
# aarch64-linux-android --> arm64-v8a
# armv7-linux-androideabi --> armeabi
# x86_64-linux-android --> x86_64
# i686-linux-android --> x86

target_triples=(aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android)
architectures=(arm64-v8a armeabi x86_64 x86)

if [ ${#target_triples[@]} != ${#architectures[@]} ]; then
    echo "Number of target_triples has to match number of architectures!"
    exit 17
fi

platform="--platform 29"
release=""
build_type=debug

if [[ $* == *--release* ]]; then
    # release builds
    release="--release"
    build_type=release
fi

for target_triple in ${target_triples[@]}; do
    cargo ndk $platform --target $target_triple build $release
done

# Linking ###########################################################

PATH_TO_ANDROID_MAIN=$root/android/core/core/src/main

echo "PATH_TO_ANDROID_MAIN is $PATH_TO_ANDROID_MAIN"

rm -fr $PATH_TO_ANDROID_MAIN/jniLibs
mkdir $PATH_TO_ANDROID_MAIN/jniLibs
for arch in ${architectures[@]}; do
    mkdir $PATH_TO_ANDROID_MAIN/jniLibs/$arch
done

lib_file=libcoepi_core.so

echo "Copying .so files to $PATH_TO_ANDROID_MAIN/jniLibs..."

cnt=${#target_triples[@]}
limit=$(($cnt - 1))

i=0
while [ "$i" -le "$limit" ]; do
    cp $root/target/${target_triples[i]}/$build_type/$lib_file $PATH_TO_ANDROID_MAIN/jniLibs/${architectures[i]}/$lib_file

    if [ $build_type = debug ]; then
        echo "Copying $build_type/$lib_file to  $PATH_TO_ANDROID_REPO/app/src/main/jniLibs/${architectures[i]}/"
        cp $root/target/${target_triples[i]}/$build_type/$lib_file $PATH_TO_ANDROID_REPO/app/src/main/jniLibs/${architectures[i]}/$lib_file
    fi
    i=$(($i + 1))
done

echo "Done"
