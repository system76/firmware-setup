#!/usr/bin/env bash

sudo apt-get install \
    --yes \
    make \
    mtools \
    parted

# Ensure the correct toolchain and components are installed
rustup show
