use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(
        "Unable to register dependency `{0}` as `DEP_{{dependency}}` already exists as a variable"
    )]
    VariableDependencyClash(String),
    #[error("Dependency `{0}` already registered")]
    DependencyAlreadyRegistered(String),
    #[error("Package `{0}` does not exist")]
    PackageDoesNotExist(String),
    #[error("Package `{0}` does not provide a binary")]
    NonBinaryPackage(String),
    #[error("Package `{0}` not built")]
    PackageNotBuilt(String),
    #[error("Package `{0}` is not installed")]
    PackageNotInstalled(String),
    #[error("Package did not provide a `name` field")]
    PackageDidNotProvideName,
    #[error("Package did not provide a `version` field")]
    PackageDidNotProvideVersion,
    #[error("Too many arguments provided. Command {0}")]
    TooManyArguments(u64),
    #[error("Not enough arguments provided. Command {0}")]
    NotEnoughArugments(u64),
    #[error("Path `{0}` does not exist")]
    PathDoesNotExist(PathBuf),
    #[error("Command `{0}` does not exist")]
    NoSuchCommand(String),
    #[error("Variable name may not contain spaces. Comand {0}")]
    VariableNameHasSpaces(u64),
    #[error("Unknown archive type. Command {0}")]
    UnknownArchiveType(u64),
    #[error("Variable `{0}` does not exist")]
    NoVariable(String),
    #[error("{0}")]
    IoErr(#[from] std::io::Error),
    #[error("The output of the script was not UTF-8")]
    ScriptOutputNotUtf8,
    #[error("Parser state corrupted")]
    ParserStateCorrupted,
    #[error("All variables are immutable. Command {0}")]
    VariableMutated(u64),
    #[error("How did we get here?")]
    HowDidWeGetHere,
    #[error("The archive type must be set before `src/srcl`")]
    ArchiveTypeNotSet,
    #[error("{0}")]
    FsExtraError(#[from] fs_extra::error::Error),
    #[error("Invalid path provided. Command {0}")]
    InvalidPath(u64),
    #[error("Archive type may only be used with file `srcl`. Command {0}")]
    TargetNotFile(u64),
    #[error("{0}")]
    ArchiveError(#[from] archive::ArchiveError),
    #[error("{0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("{0}")]
    VarError(#[from] std::env::VarError),
    #[error("{0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    TomlDeError(#[from] toml::de::Error),
    #[error("{0}")]
    TomlSerError(#[from] toml::ser::Error),
    #[error("Failed to convert an OS string into a normal string")]
    OsStrToStrConversionError,
}
