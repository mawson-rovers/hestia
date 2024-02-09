#!/bin/bash

FILE=$1
if [ -z "$FILE" ]; then
  echo "Usage: $0 file"
  exit 1
fi
PATH=$(dirname "$FILE")

HOST=beagle  # map this to the actual host name in your ~/.ssh/config
DEST=/home/debian/uts

set -x # echo commands from now on
/usr/bin/ssh "$HOST" "mkdir -p $DEST/bin/"
/usr/bin/rsync -utvz -e /usr/bin/ssh "$PATH"/uts-{cli,log,web,run,update} "$HOST:$DEST/bin/"
/usr/bin/rsync -utvzr -e /usr/bin/ssh ./{nginx,systemd,programs} "$HOST:$DEST/"
/usr/bin/rsync -utvzr -e /usr/bin/ssh ../hestia-static-dash/ "$HOST:$DEST/www/"
