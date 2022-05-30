#!/bin/bash

docker run -i \
    -v $HOME/src/mawson/hestia:/local/src \
    -v /Applications/KiCad/KiCad.app/Contents/SharedSupport:/local/kicad \
    -v $HOME/Library/Preferences/kicad:/root/.config/kicad \
    -e 'KICAD6_3DMODEL_DIR=/local/kicad/3dmodels' \
    -w /local/src \
    -t setsoft/kicad_auto_test:ki6 \
    ./scripts/kibot-build.sh
 #        /bin/bash  # use for debugging
