#!/usr/bin/env bash

bash ./build.sh

cargo test --test escrow-tests -- --nocapture