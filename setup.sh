#!/bin/sh

# Work around some dumbassery in rust where it's impossible to set a custom target without
# fiddling with the environment
# You need to `source` this script for it to work
export RUST_TARGET_PATH=$PWD
