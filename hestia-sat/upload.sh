#!/bin/bash

DEST=$1
FILE=$2
if [ -z "$DEST" ] || [ -z "$FILE" ]; then
  echo "Usage: $0 host:path/ file"
  exit 1
fi
PATH=$(dirname "$FILE")

set -x # echo commands from now on
/usr/bin/scp "$PATH"/{uts-cli,uts-log,uts-web} "$DEST"
