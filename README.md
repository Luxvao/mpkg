# Mpkg
Mpkg is a minimal package manager/auto-updater utility.

## Features
- Basic dependency management
- Checking for updates
- Launching binary packages
- Provides build and install directories

## How does it work?
It operates on packages that are placed at `~/.mpkg/packages/<package name>/`. Every package has its own `build.mpkg` definition file. Here is an example of one that I use for Ghidra:
```
# Basic metadata
name Ghidra
description Advanced Open-Source SRE
bin ghidra_wrapper.sh

# Dynamic (autoupdate)
setsh VERSION, fetch_version.sh
setsh ZIP_NAME, fetch_zip.sh

# Dependencies
dep java_openjdk26

# Version
version %VERSION

# Source stuff
archive_type zip
src https://github.com/NationalSecurityAgency/ghidra/releases/download/%VERSION/%ZIP_NAME

archive_type other
srcl ghidra_wrapper.sh

# Build
build build.sh
```

These files are executed procedurally and therefore have somewhat strict requirements on what goes where. The most notable of these is the "`archive_type` before `src`/`srcl`".
Since the interpreter executes `src` and `srcl` commands as it comes across them, it has to know what to actually do with them. That's why it's required to set the archive type
before executing a fetch source command. It's important to remember that multiple sources are supported, but `archive_type` must be called again at some point before the next one.
That is to ensure correctness. While the context could store the type through the entire build process, it to me seems reasonable not to do so as it could lead to implicit archive type
issues that otherwise do not happen. I have therefore decided to simply error and abort upon encountering a "bare" source directive (no archive type provided). For more information please check
out `crates/mpkg_core/docs/language_spec.md` which defines every command and their behaviour (it covers path nuances too).

Packages must at minimum provide a:
- `name`
- `version`

## What about auto-updates?
Mpkg by design refuses to provide a centralised repository. It to me is not useful, nor is it something I wish to maintain. Therefore I've thought of the `setsh` system. Mpkg supports
2 types of variables. We have static variables (`set VARIABLE, whatever`) which are evaluated once and are (as the name implies) static. Then there are dynamic variables. These are the core of
the auto-update system. For a dynamic variable, you will need a shell script. Technically that's a lie. You simply need an executable file that will give some output. That output is the contents
of the dynamic variable. To illustrate: `setsh VARIABLE, fetch_version.sh` will create a new dynamic variable named `VARIABLE`. It will eagerly evaluate `fetch_version.sh` (relative to the package directory)
and store its output for future reference. When checking for updates, it will compare the previous output of the script to the new one. If there is a difference, then the package by convention has an update
available.

I'd suggest looking at `examples` for a peek into my `~/.mpkg/packages/` (as of writing this).

