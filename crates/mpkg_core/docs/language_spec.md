# `mpkg` Language Specification
`mpkg` provides a minimal scripting language to create recipes with.

## Basics
- Commands must terminate with `\n`
- All `setsh` commands will be rerun on every launch. If contents change the user will be prompted as to whether they'd wish to rerun the building of the package (this facilitates auto updating)
- `%<variable name>` represents a variable substitution in `mpkg`. Variables may not have spaces. They are made available within the shell scripts as well under `$<variable name>`

## Conventions
- `set`, `setsh`, `srcl`, `build` and `uninstall` use a path relative to the package directory
- `bin` uses a path relative to the install directory

## Caveats
Because `setsh` is meant to be used for update checking and general checks, it must be self contained. Therefore substitution in a `setsh` is not permitted (substitution will simply not work). If needed, prepare everything beforehand with a `build` (there may be multiple `build`s present in a single package definition)

## Commands
`set <variable name>, <string>` - Instantiates a variable `<variable name>` with the contents set to the passed string

`setsh <variable name>, <path>` - Instantiates a variable `<variable name>` with the contents set to the `stdout` output of the shell script at `<path>` (path relative to package dir)

`name <string>` - Sets the name of the package

`description <string>` - Sets the description of the package

`version <string>` - Sets the version of the package

`src <url>` - URL to an archive of the source

`srcl <path>` - Local path to source

`dep <package>` - Defines a dependency. Dependencies are available at build-time and runtime at `$DEP_{dependency (in capitals)}`

`archive_type <zip|tar|other>` - Sets the type of archive pointed to by `src/srcl`. `other` is used for "exotic" (or not-so-exotic) archives handled in `build` (downloaded sources of archive_type `other` are downloaded as `BUILD_DIR/downloaded` so make sure to rename them with a `build` in-between if pulling multiple sources)

`bin <path>` - Sets the binary file to launch

`build <path>` - Provides a shell script that is ran in `BUILD_DIR` to build and clean the project before the entire build directory is moved to `INSTALL_DIR`

`uninstall <path>` - Provides an uninstall shell script that is ran in `INSTALL_DIR` to undo whatever the build script had done
