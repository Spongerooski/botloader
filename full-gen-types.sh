#!/bin/bash
set -e

# cd to the script location
cd "${0%/*}"

./components/runtime-models/gen-move-types.bash
