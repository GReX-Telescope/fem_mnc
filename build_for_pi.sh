#!/usr/bin/env bash

TARGET=armv7-unknown-linux-gnueabihf # Pi 3 and 4
TARGET_HOST=snap-pi # Configured in .ssh/config
TARGET_PATH=/home/pi/fem_mnc

export CROSS_CONTAINER_ENGINE=podman # Or docker

cross build --release --target $TARGET
scp target/armv7-unknown-linux-gnueabihf/release/fem_mnc ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} RUST_LOG=TRACE ${TARGET_PATH}
