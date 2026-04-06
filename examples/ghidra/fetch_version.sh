#! /bin/sh

curl -s https://api.github.com/repos/NationalSecurityAgency/ghidra/releases/latest | jq -r '.tag_name'
