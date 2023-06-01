#!/bin/sh

DEST=$1
FILE=$2
if [ -z "$DEST" ] || [ -z "$FILE" ]; then
  echo "Usage: $0 file host:path/"
  exit 1
fi
scp "$FILE" "$DEST"
