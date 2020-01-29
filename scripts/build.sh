#!/usr/bin/env sh

set -e

export CARGO_INCREMENTAL=0

for SRC in runtime/wasm
do
  echo "Building webassembly binary in $SRC..."
  cd "$SRC"

  ./build.sh

  cd - >> /dev/null
done