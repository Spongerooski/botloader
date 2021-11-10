#!/bin/bash
tsc --build tsconfig.json
cp lib.botloader_user.core.d.ts ../../../../typings
cp script_globals.d.ts ../../../../typings