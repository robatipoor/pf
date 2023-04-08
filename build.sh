#!/usr/bin/env bash

set -euxo pipefail

if ! [ -x "$(command -v upx)" ]; then
  echo >&2 "Error: upx is not installed."
  exit 1
fi

TARGET="x86_64-unknown-linux-gnu"
echo "*** build project target: $TARGET ***"
cargo build --target $TARGET --release
echo "*** compress server binary ***"
upx target/$TARGET/release/server
echo "*** compress client binary ***"
upx target/$TARGET/release/client
