#!/bin/bash

# Runs Hestia kibot build (normally in a Docker container). Prerequisites:
# - working directory should be top level of source directory
# - kibot should be on the PATH

PCB_PATH="$PWD/hestia-pcb"
BUILD_PATH="$PWD/build/hestia-pcb"

set -e  # abort if any command fails

echo "[INFO] Starting build script $0"
echo "[INFO] - PCB path: $PCB_PATH"
echo "[INFO] - Build output: $BUILD_PATH"

pushd $PCB_PATH  # do this first, so we fail if we're in the wrong spot

echo "[INFO] Deleting old build files"
[ -d "$BUILD_PATH" ] && rm -r "$BUILD_PATH"

if [ -z "$KICAD7_3DMODEL_DIR" ]; then
    echo "[INFO] Installing 3D models"
    /usr/bin/kicad_3d_install.sh
fi

# Ensure we set KICAD7_3DMODEL_DIR to avoid lots of build errors
export KICAD7_3DMODEL_DIR=${KICAD7_3DMODEL_DIR:-'/usr/share/kicad/3dmodels'}
echo "[INFO] KICAD7_3DMODEL_DIR set to $KICAD7_3DMODEL_DIR"

CONFIG=hestia.kibot.yml
echo "[INFO] Starting kibot with config: $CONFIG"
kibot -c "$CONFIG" -b hestia.kicad_pcb -e hestia.kicad_sch \
      -d "$BUILD_PATH"
popd

echo "[INFO] Finished build script $0"
