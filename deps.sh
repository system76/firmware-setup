#!/usr/bin/env bash

sudo apt-get install \
    --yes \
    make \
    mtools \
    parted

# Ensure the correct toolchain is installed
rustup show
# Install required components
rustup component add rust-src
