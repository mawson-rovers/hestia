#!/bin/bash

VERSION=$1
if [ -z "$VERSION" ]; then
  echo "Usage: $0 v2.2"
  exit 1
fi

OUT="target/release"
TARGET="arm-unknown-linux-gnueabihf"
ARCHIVE="target/hestia_sat-$VERSION-$TARGET.tar.gz"

set -x # echo commands from now on
rm -r $OUT
mkdir -p $OUT/{bin,www,nginx,systemd}

cp -p "target/$TARGET/debug/"uts-{cli,log,web,run} $OUT/bin/
cp -p ./uts-programs.toml $OUT/uts-programs-example.toml
cp -Rp ./{nginx,systemd} $OUT/
cp -Rp ../hestia-static-dash/* $OUT/www/

tar zcf "$ARCHIVE" --strip-components 2 --exclude '.DS_Store' $OUT

