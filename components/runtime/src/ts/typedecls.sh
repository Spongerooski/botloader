#!/bin/bash
set -e

# cd to the script location
cd "${0%/*}"

rm -r typings

tsc --build tsconfig.json
cp lib.botloader_user.core.d.ts typings
cp script_globals.d.ts typings