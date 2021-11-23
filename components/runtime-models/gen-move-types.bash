#!/bin/bash
set -e

# cd to the script location
cd "${0%/*}"

# generate the types
cargo t

# generate indexes
./gen-index.bash

# move bindings
rm -r ../runtime/src/ts/models
mv bindings/ ../runtime/src/ts/models