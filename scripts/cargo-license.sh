#!/bin/bash
set -xe
if [ ! -e src-tauri/about.hbs ]; then
    (cd src-tauri && cargo.exe about init)
fi
(cd src-tauri && cargo.exe about generate -o ../src/components/resource/THIRD-PARTY-NOTICES-cargo.txt about.hbs)
