#!/bin/bash

PROJECTS="get-nico-data html-gen merge-nico-data merge-rankings sort-ranking nico-ranking"

for rank in $PROJECTS; do
  # shellcheck disable=SC2086
  cargo uinstall "$rank"
done
