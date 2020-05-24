#!/bin/bash

PATH_TO_LIB_REPO="/Users/john/Code/Co-Ep/app-backend-rust"
PATH_TO_ANDROID_REPO="/Users/john/Code/Co-Ep/app-android"

PATH_TO_ANDROID_MAIN=$PATH_TO_ANDROID_REPO/app/src/main

mkdir $PATH_TO_ANDROID_MAIN/jniLibs
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/arm64
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/armeabi
mkdir $PATH_TO_ANDROID_MAIN/jniLibs/x86

ln -s $PATH_TO_LIB_REPO/target/aarch64-linux-android/release/libtcn_client.so $PATH_TO_ANDROID_MAIN/jniLibs/arm64/libtcn_client.so
ln -s $PATH_TO_LIB_REPO/target/i686-linux-android/release/libtcn_client.so $PATH_TO_ANDROID_MAIN/jniLibs/x86/libtcn_client.so
ln -s $PATH_TO_LIB_REPO/target/armv7-linux-androideabi/release/libtcn_client.so $PATH_TO_ANDROID_MAIN/jniLibs/armeabi/libtcn_client.so
