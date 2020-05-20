# app-backend-rust

WIP - iOS/Android shared Rust library

Helpers wanted - if you have Rust experience for iOS/Android app integration, please reach out!


## Currently exposed functions:
- get_reports
- post_report

## Build

Install [rustup](https://rustup.rs/)

### iOS

Add target

> rustup target add x86_64-apple-ios

Update script with your PATH_TO_IOS_REPO value and run:

> ./scripts/ios/create_ios_universal.sh

Update script with your OUTPUT_DIR value and run:

> ./scripts/ios/make_framework.sh

### Android 

Add targets

> rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android

Update script with your PATH_TO_ANDROID_REPO value and run:

> ./scripts/ios/create_android_targets.sh

Update script with your OUTPUT_DIR value and run:

> ./scripts/ios/make_framework.sh