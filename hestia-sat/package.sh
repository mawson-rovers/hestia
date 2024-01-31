#!/bin/bash

VERSION=$1
if [ -z "$VERSION" ]; then
  echo "Usage: $0 v2.2"
  exit 1
fi

OUT="target/package"
TARGET="arm-unknown-linux-gnueabihf"
EARTH_ARCHIVE="target/uts-matilda-$VERSION-earth.tar.gz"
SPACE_ARCHIVE="target/uts-matilda-$VERSION-space.tar.gz"

export COPYFILE_DISABLE=1  # On Mac, don't add ._ resource fork files in tarballs

set -x # echo commands from now on

echo "Building test archive"
rm -r $OUT
mkdir -p $OUT/{bin,www,programs}

cp -p "target/$TARGET/release/"uts-{cli,log,web,run,update} $OUT/bin/
cp -p programs/*.toml $OUT/programs/
cp -Rp ../hestia-static-dash/* $OUT/www/

tar zcf "$EARTH_ARCHIVE" --no-xattrs --strip-components 2 --exclude '.DS_Store' $OUT

echo "Building prod archive"
rm -r $OUT
mkdir -p $OUT/{bin,programs,kubos}

cp -p "target/$TARGET/release/"uts-{cli,log,run} $OUT/bin/
cp -p programs/*.toml $OUT/programs/
cp -R kubos/* $OUT/kubos/
ln -s "../../bin/uts-cli" $OUT/kubos/uts-cli/
ln -s "../../bin/uts-log" $OUT/kubos/uts-log/
ln -s "../../bin/uts-run" $OUT/kubos/uts-run/

tar zcf "$SPACE_ARCHIVE" --no-xattrs --strip-components 2 --exclude '.DS_Store' $OUT
