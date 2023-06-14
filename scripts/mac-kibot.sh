#!/bin/bash

COMMAND="./scripts/kibot-build.sh"
if [[ "$1" == "-d" || "$1" == "--debug" ]]; then
    COMMAND="/bin/bash"
fi

docker run --platform=linux/amd64 -i \
    -v $HOME/src/mawson/hestia:/local/src \
    -v /Applications/KiCad/KiCad.app/Contents/SharedSupport:/local/kicad \
    -v $HOME/Library/Preferences/kicad:/root/.config/kicad \
    -e 'KICAD6_3DMODEL_DIR=/local/kicad/3dmodels' \
    -w /local/src \
    -t ghcr.io/inti-cmnb/kicad7_auto:1.6.1 \
    "$COMMAND"
