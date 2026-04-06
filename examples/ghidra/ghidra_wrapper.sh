#! /bin/sh

# Java fix + java home
_JAVA_AWT_WM_NONREPARENTING=1 JAVA_HOME="$DEP_JAVA_OPENJDK26" ./ghidraRun
