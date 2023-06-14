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
/usr/bin/rsync -utvz -e /usr/bin/ssh "$PATH"/{uts-cli,uts-log,uts-web} "$HOST:$DEST/bin/"
/usr/bin/rsync -utvzr -e /usr/bin/ssh ./{nginx,systemd} "$HOST:$DEST/"
