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
    -t setsoft/kicad_auto:ki6.0.5_Debian \
    "$COMMAND"
