#!/bin/bash

# Runs Hestia kibot build (normally in a Docker container). Prerequisites:
# - working directory should be top level of source directory
# - kibot should be on the PATH

PCB_PATH="$PWD/hardware/pcb/hestia"
BUILD_PATH="$PWD/build/pcb"

set -e  # abort if any command fails

echo "[INFO] Starting build script $0"
echo "[INFO] - PCB path: $PCB_PATH"
echo "[INFO] - Build output: $BUILD_PATH"

pushd $PCB_PATH  # do this first, so we fail if we're in the wrong spot

echo "[INFO] Deleting old build files"
[ -d "$BUILD_PATH" ] && rm -r "$BUILD_PATH"

echo "[INFO] Starting kibot with config: hestia.kibot.yml"
kibot -c hestia.kibot.yml -b hestia.kicad_pcb -e hestia.kicad_sch \
        -d "$BUILD_PATH"
popd

echo "[INFO] Finished build script $0"
