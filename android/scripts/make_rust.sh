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

# cd android/core || exit 1

platform="--platform 29"
target_triples=(aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android)
architectures=(arm64-v8a armeabi x86_64 x86)
echo ${target_triples[*]}
echo ${target_triples[0]} ${architectures[0]}
echo ${target_triples[3]} ${architectures[3]}

release=""

if [[ $* == *--release* ]]; then
    # release builds
    release="--release"
    #cargo ndk $platform --target x86_64-linux-android build --release
    #cargo ndk --platform 29 --target aarch64-linux-android build --release
    #cargo ndk --platform 29 --target armv7-linux-androideabi build --release
    #cargo ndk --platform 29 --target i686-linux-android build --release
# else
    # debug builds
    #cargo ndk $platform --target x86_64-linux-android build
    #cargo ndk --platform 29 --target aarch64-linux-android build
    #cargo ndk --platform 29 --target armv7-linux-androideabi build
    #cargo ndk --platform 29 --target i686-linux-android build
fi

for target_triple in ${target_triples[@]}
do
echo "cargo ndk $platform --target $target_triple build $release"
done



# cd ../.. || exit 2

# Linking ###########################################################

PATH_TO_ANDROID_MAIN=$root/android/core/core/src/main

echo "PATH_TO_ANDROID_MAIN is $PATH_TO_ANDROID_MAIN"

#rm -fr $PATH_TO_ANDROID_MAIN/jniLibs
#mkdir $PATH_TO_ANDROID_MAIN/jniLibs
for arch in ${architectures[@]}
do
echo "mkdir $PATH_TO_ANDROID_MAIN/jniLibs/$arch"
done


#mkdir $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a
#mkdir $PATH_TO_ANDROID_MAIN/jniLibs/armeabi
#mkdir $PATH_TO_ANDROID_MAIN/jniLibs/x86_64
#mkdir $PATH_TO_ANDROID_MAIN/jniLibs/x86

build_type=debug
lib_file=libcoepi_core.so

if [[ $* == *--release* ]]; then
    # release
    build_type=release
    cp $root/target/aarch64-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a/libcoepi_core.so
    #cp $root/target/x86_64-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86_64/libcoepi_core.so
    #cp $root/target/armv7-linux-androideabi/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/armeabi/libcoepi_core.so
    #cp $root/target/i686-linux-android/release/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86/libcoepi_core.so
else
    # debug
    echo "Copying .so files to $PATH_TO_ANDROID_MAIN/jniLibs..."
    echo "cp $root/target/aarch64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64-v8a/libcoepi_core.so"
    echo "cp $root/target/${target_triples[0]}/$build_type/$lib_file $PATH_TO_ANDROID_MAIN/jniLibs/${architectures[0]}/$lib_file"
    
    echo "cp $root/target/armv7-linux-androideabi/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/armeabi/libcoepi_core.so"
    echo "cp $root/target/${target_triples[1]}/$build_type/$lib_file $PATH_TO_ANDROID_MAIN/jniLibs/${architectures[1]}/$lib_file"
    
    echo "cp $root/target/x86_64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86_64/libcoepi_core.so"
    echo "cp $root/target/${target_triples[2]}/$build_type/$lib_file $PATH_TO_ANDROID_MAIN/jniLibs/${architectures[2]}/$lib_file"
    
    echo "cp $root/target/i686-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_MAIN/jniLibs/x86/libcoepi_core.so"
    echo "cp $root/target/${target_triples[3]}/$build_type/$lib_file $PATH_TO_ANDROID_MAIN/jniLibs/${architectures[3]}/$lib_file"

    exit 0

    echo "Copying .so files to  $PATH_TO_ANDROID_REPO/app/src/main/jniLibs..."
    cp $root/target/aarch64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_REPO/app/src/main/jniLibs/arm64-v8a/libcoepi_core.so
    #cp $root/target/x86_64-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_REPO/app/src/main/jniLibs/x86_64/libcoepi_core.so
    #cp $root/target/armv7-linux-androideabi/debug/libcoepi_core.so $PATH_TO_ANDROID_REPO/app/src/main/jniLibs/armeabi/libcoepi_core.so
    #cp $root/target/i686-linux-android/debug/libcoepi_core.so $PATH_TO_ANDROID_REPO/app/src/main/jniLibs/x86/libcoepi_core.so
fi

echo "Done"