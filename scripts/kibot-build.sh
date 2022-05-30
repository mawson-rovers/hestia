#!/bin/bash

# Runs Hestia kibot build (normally in a Docker container). Prerequisites:
# - working directory should be top level of source directory
# - kibot should be on the PATH

set -e  # abort if any command fails

echo "[INFO] Starting build script $0"

pushd hardware/pcb/hestia  # do this first, so we fail if we're in the wrong spot

echo "[INFO] Deleting old build files"
rm -r ../../../build

echo "[INFO] Starting kibot with config: hestia.kibot.yml"
kibot -c hestia.kibot.yml -b hestia.kicad_pcb -e hestia.kicad_sch \
        -d ../../../build
popd

echo "[INFO] Finished build script $0"
