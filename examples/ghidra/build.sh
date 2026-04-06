#! /bin/sh

GHIDRA_DIR="$(echo "$ZIP_NAME" | sed 's/C.*\.zip//')C"

# Fix ghidra stuff
mv ${GHIDRA_DIR}/* .
rmdir "${GHIDRA_DIR}"
chmod +x ghidraRun
chmod +x support/launch.sh
