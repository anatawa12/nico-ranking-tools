#!/bin/bash

TARGET="$1"

set -eu

if [ -z "$TARGET" ]; then
  TARGET_PARAM=""
else
  TARGET_PARAM="--target $TARGET"
fi

PROJECTS="get-nico-data html-gen merge-nico-data merge-rankings sort-ranking nico-ranking"

mkdir -p dist

for rank in $PROJECTS; do
  # shellcheck disable=SC2086
  cargo build --manifest-path="$rank/Cargo.toml" $TARGET_PARAM --release
  built_path_file="$(cargo metadata --manifest-path="$rank/Cargo.toml" --format-version 1 | jq -r '.target_directory')/$TARGET/release/$rank.d"
  built_binary=$(head -n 1 "$built_path_file" | cut -d: -f1)
  cp "$built_binary" ./dist/
done
