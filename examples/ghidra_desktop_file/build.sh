#! /bin/sh

cat << EOF > "/home/$USER/.local/share/applications/ghidra.desktop"
[Desktop Entry]
Version=1.0
Type=Application
Name=Ghidra
Exec=mpkg launch ghidra
Icon=${DEP_GHIDRA}/support/ghidra.ico
Categories=Development;ReverseEngineering;
StartupNotify=true
Terminal=false
EOF
