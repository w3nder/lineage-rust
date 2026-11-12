#!/bin/sh
set -eu
TARGET="${1:-aarch64-apple-ios-sim}"
PROFILE=release
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export CARGO_INCREMENTAL=0
cargo build -j2 --release --target "$TARGET" -p killer --manifest-path "$ROOT/Cargo.toml"
APP="$ROOT/target/ios/Killer.app"
rm -rf "$APP"
mkdir -p "$APP"
cp "$ROOT/target/$TARGET/$PROFILE/killer" "$APP/killer"
cp "$ROOT/apps/killer/ios/Info.plist" "$APP/Info.plist"
codesign --force --sign "${IOS_SIGN_IDENTITY:--}" "$APP"
echo "$APP"
