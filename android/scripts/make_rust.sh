#!/bin/bash 

# to debug the script add " -x" to the shebang. Ie. "#!/bin/bash -x"

# get script folder
folder=$( cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P )

echo "$folder"

# get android main folder relative to script folder
parent="$(dirname "$folder")"
PATH_TO_ANDROID_MAIN="$parent/core/core/src/main"
echo "PATH_TO_ANDROID_MAIN is $PATH_TO_ANDROID_MAIN"

# get project root
root="$(dirname "$parent")"
echo "Root folder: $root"


# Run from repo's root folder (relative to Gradle build file. TODO do this in Gradle)

# Building  ###########################################################

# See https://github.com/Co-Epi/app-backend-rust/wiki/Building-library-for-Android

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
    i=$(($i + 1))
done

echo "Done"
