use std::{collections::HashMap, path::PathBuf};

use archive::ArchiveFormat;
use fs_extra::dir::CopyOptions;
use serde::{Deserialize, Serialize};

use crate::{
    directory::get_package_dirs,
    error::Error,
    package::{build_package, check_package_update},
    util::{download_with_progress, extract_archive},
};

macro_rules! parse {
    ($parser:ident, $command_str:expr, $command_no:expr) => {
        $parser($command_str.nth(0).unwrap_or_default(), $command_no)
    };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Command {
    Set(Variable),
    Setsh(Variable),
    Name(String),
    Description(String),
    Version(String),
    Src(String),
    Srcl(String),
    Dep(String),
    ArchiveType(ArchiveType),
    Bin(String),
    Build(String),
    Uninstall(String),
    NoOp,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Variable {
    Static {
        name: String,
        value: String,
    },
    Dynamic {
        name: String,
        shell_script: PathBuf,
        value: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dependency {
    pub package: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum ArchiveType {
    Zip,
    Tar,
    Other,
}

#[derive(Default, Clone, Debug)]
pub struct Context {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub bin: Option<PathBuf>,
    pub uninstall: Option<PathBuf>,
    archive_type: Option<ArchiveType>,
    pub variables: HashMap<String, Variable>,
    pub dependencies: HashMap<String, Dependency>,
}

#[derive(Clone, Debug)]
pub struct PackageContext {
    pub build_dir: PathBuf,
    pub install_dir: PathBuf,
    pub package_dir: PathBuf,
}

impl Command {
    pub fn from_str(command: &str, command_no: u64) -> Result<Command, Error> {
        let mut command_str = command.splitn(2, " ");

        match command_str.nth(0).map(|s| s.trim()) {
            Some("set") => parse!(set, command_str, command_no),
            Some("setsh") => parse!(setsh, command_str, command_no),
            Some("name") => parse!(name, command_str, command_no),
            Some("description") => parse!(description, command_str, command_no),
            Some("version") => parse!(version, command_str, command_no),
            Some("src") => parse!(src, command_str, command_no),
            Some("srcl") => parse!(srcl, command_str, command_no),
            Some("dep") => parse!(dep, command_str, command_no),
            Some("archive_type") => parse!(archive_type, command_str, command_no),
            Some("bin") => parse!(bin, command_str, command_no),
            Some("build") => parse!(build, command_str, command_no),
            Some("uninstall") => parse!(uninstall, command_str, command_no),
            // Comments
            Some("#") => Ok(Command::NoOp),
            Some("") => Ok(Command::NoOp),
            Some(e) => Err(Error::NoSuchCommand(e.to_string())),
            None => Ok(Command::NoOp),
        }
    }

    pub fn evaluate(
        &self,
        ctx: &mut Context,
        package_context: &PackageContext,
        command_no: u64,
        headless: bool,
    ) -> Result<(), Error> {
        let ctx_variables = ctx.variables.values().collect::<Vec<&Variable>>();

        match self {
            Command::Set(var) => {
                let name = var.get_name();
                let value = substitute(var.get_value()?, &ctx_variables)?;

                if !headless {
                    println!("`{}` instantiated...", name);
                }

                let variable = Variable::Static {
                    name: name.clone(),
                    value,
                };

                if let Some(_) = ctx.variables.insert(name, variable) {
                    return Err(Error::VariableMutated(command_no));
                }
            }
            Command::Setsh(var) => match var {
                Variable::Dynamic {
                    name,
                    shell_script: _,
                    value: _,
                } => {
                    let mut variable = var.clone();

                    if !headless {
                        println!("`{}` instantiated [DYNAMIC]...", name);
                    }

                    // Resolve it
                    variable.resolve(ctx, package_context)?;

                    if let Some(_) = ctx.variables.insert(name.clone(), variable) {
                        return Err(Error::VariableMutated(command_no));
                    }
                }
                _ => return Err(Error::ParserStateCorrupted),
            },
            Command::Name(name) => ctx.name = Some(substitute(name.clone(), &ctx_variables)?),
            Command::Description(desc) => {
                ctx.description = Some(substitute(desc.clone(), &ctx_variables)?)
            }
            Command::Version(ver) => ctx.version = Some(substitute(ver.clone(), &ctx_variables)?),
            Command::ArchiveType(at) => ctx.archive_type = Some(at.clone()),
            Command::Src(src) => {
                if let None = ctx.archive_type {
                    return Err(Error::ArchiveTypeNotSet);
                }

                let src_substituted = substitute(src.clone(), &ctx_variables)?;

                if !headless {
                    println!("Pulling `{}`...", src_substituted);
                }

                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;

                let bytes =
                    rt.block_on(async { download_with_progress(&src_substituted).await })?;

                let archive_type = match ctx.archive_type {
                    Some(ArchiveType::Zip) => ArchiveFormat::Zip,
                    Some(ArchiveType::Tar) => ArchiveFormat::Tar,
                    Some(ArchiveType::Other) => {
                        // We just save it as $BUILD_DIR/downloaded
                        let mut target_path = package_context.build_dir.clone();
                        target_path.push("downloaded");

                        std::fs::write(target_path, &bytes)?;

                        return Ok(());
                    }
                    None => unreachable!(),
                };

                ctx.archive_type = None;

                extract_archive(&bytes, archive_type, &package_context.build_dir, headless)?;
            }
            Command::Srcl(srcl) => {
                if let None = ctx.archive_type {
                    return Err(Error::ArchiveTypeNotSet);
                }

                // Substitute before continuing
                let mut src = package_context.package_dir.clone();
                src.push(substitute(srcl.clone(), &ctx_variables)?);

                if !headless {
                    println!(
                        "Copying `{}`...",
                        src.to_str().ok_or(Error::HowDidWeGetHere)?
                    );
                }

                let name = src
                    .file_name()
                    .ok_or(Error::InvalidPath(command_no))?
                    .to_str()
                    .ok_or(Error::InvalidPath(command_no))?
                    .to_owned();

                fs_extra::copy_items(&[src], &package_context.build_dir, &CopyOptions::new())?;

                let mut target = package_context.build_dir.clone();
                target.push(name);

                let archive_type = match ctx.archive_type {
                    Some(ArchiveType::Zip) => ArchiveFormat::Zip,
                    Some(ArchiveType::Tar) => ArchiveFormat::Tar,
                    Some(ArchiveType::Other) => return Ok(()),
                    None => unreachable!(),
                };

                ctx.archive_type = None;

                if !target.is_file() {
                    return Err(Error::TargetNotFile(command_no));
                }

                let file_contents = std::fs::read(target)?;

                extract_archive(
                    &file_contents,
                    archive_type,
                    &package_context.build_dir,
                    headless,
                )?;
            }
            Command::Dep(dep) => {
                let dep = substitute(dep.clone(), &ctx_variables)?;

                if !headless {
                    println!("Registered `{}` as a dependency...", dep);
                }

                match check_package_update(&dep) {
                    Err(Error::PackageNotBuilt(_)) => {
                        if !headless {
                            println!("Building dependency `{}`...", dep);
                        }

                        build_package(&dep)?
                    }
                    Err(e) => return Err(e),
                    _ => (),
                }

                // Instantiate a dep object
                let dependency = Dependency {
                    package: dep.clone(),
                };

                // Variable stuff
                let var_name = dependency.get_dep_var_name();
                let var_value = dependency.get_dep_install_dir()?;

                let var = Variable::Static {
                    name: var_name.clone(),
                    value: var_value,
                };

                // Register
                if let Some(_) = ctx
                    .dependencies
                    .insert(dependency.get_dep().to_string(), dependency)
                {
                    return Err(Error::DependencyAlreadyRegistered(dep));
                }

                if let Some(_) = ctx.variables.insert(var_name, var) {
                    return Err(Error::VariableDependencyClash(dep));
                }
            }
            Command::Bin(bin) => {
                let mut bin_path = package_context.install_dir.clone();
                bin_path.push(substitute(bin.clone(), &ctx_variables)?);

                ctx.bin = Some(bin_path);
            }
            Command::Build(build) => {
                // Substitute
                let build = substitute(build.clone(), &ctx_variables)?;

                if !headless {
                    println!("Running `{}`...", build);
                }

                let mut build_script_path = package_context.package_dir.clone();
                build_script_path.push(build);

                let ctx_variables_env = ctx
                    .variables
                    .iter()
                    .map(|(name, var)| -> Result<(String, String), Error> {
                        Ok((name.clone(), var.get_value()?))
                    })
                    .collect::<Result<Vec<(String, String)>, Error>>()?;

                std::process::Command::new(build_script_path)
                    .current_dir(&package_context.build_dir)
                    .env("BUILD_DIR", package_context.build_dir.as_os_str())
                    .env("INSTALL_DIR", package_context.install_dir.as_os_str())
                    .env("PACKAGE_DIR", package_context.package_dir.as_os_str())
                    .envs(ctx_variables_env)
                    .status()?;
            }
            Command::Uninstall(uninstall) => {
                let mut uninstall_path = package_context.package_dir.clone();
                uninstall_path.push(substitute(uninstall.clone(), &ctx_variables)?);

                ctx.uninstall = Some(uninstall_path);
            }
            Command::NoOp => (),
        }

        Ok(())
    }
}

impl Variable {
    pub fn resolve(
        &mut self,
        context: &mut Context,
        package_context: &PackageContext,
    ) -> Result<(), Error> {
        if let Variable::Dynamic {
            name: _,
            shell_script,
            value,
        } = self
        {
            // Evaluate the shell script
            let mut shell_script_path = package_context.package_dir.clone();
            shell_script_path.push(shell_script);

            let ctx_variables = context
                .variables
                .iter()
                .map(|(name, var)| -> Result<(String, String), Error> {
                    Ok((name.clone(), var.get_value()?))
                })
                .collect::<Result<Vec<(String, String)>, Error>>()?;

            let output = std::process::Command::new(shell_script_path)
                .current_dir(&package_context.build_dir)
                .env("BUILD_DIR", package_context.build_dir.as_os_str())
                .env("INSTALL_DIR", package_context.install_dir.as_os_str())
                .env("PACKAGE_DIR", package_context.package_dir.as_os_str())
                .envs(ctx_variables)
                .output()?;

            let output_string =
                String::from_utf8(output.stdout).map_err(|_| Error::ScriptOutputNotUtf8)?;

            *value = Some(output_string);
        }

        Ok(())
    }

    pub fn is_dynamic(&self) -> bool {
        match self {
            Variable::Dynamic {
                name: _,
                shell_script: _,
                value: _,
            } => true,
            _ => false,
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Variable::Static { name, value: _ } => name.clone(),
            Variable::Dynamic {
                name,
                shell_script: _,
                value: _,
            } => name.clone(),
        }
    }

    pub fn get_value(&self) -> Result<String, Error> {
        match self {
            Variable::Static { name: _, value } => Ok(value.clone()),
            Variable::Dynamic {
                name: _,
                shell_script: _,
                value,
            } => value.clone().ok_or(Error::ParserStateCorrupted),
        }
    }
}

impl Dependency {
    pub fn get_dep(&self) -> &str {
        &self.package
    }

    pub fn get_dep_var_name(&self) -> String {
        format!("DEP_{}", self.package.to_uppercase())
    }

    pub fn get_dep_install_dir(&self) -> Result<String, Error> {
        let (_, install_dir) = get_package_dirs(&self.package)?;

        Ok(install_dir
            .to_str()
            .ok_or(Error::HowDidWeGetHere)?
            .to_string())
    }
}

fn substitute(mut input: String, vars: &[&Variable]) -> Result<String, Error> {
    for var in vars {
        let pattern = format!("%{}", var.get_name());

        input = input.replace(pattern.as_str(), &var.get_value()?.trim());
    }

    Ok(input)
}

fn empty_string_chk(input: &str, command_no: u64) -> Result<(), Error> {
    if input.is_empty() {
        return Err(Error::NotEnoughArugments(command_no));
    }

    Ok(())
}

fn set(arguments: &str, command_no: u64) -> Result<Command, Error> {
    let mut arguments = arguments.splitn(2, ",");

    let name = arguments
        .nth(0)
        .ok_or(Error::NotEnoughArugments(command_no))?
        .trim();

    let value = arguments
        .nth(0)
        .ok_or(Error::NotEnoughArugments(command_no))?
        .trim();

    if name.contains(" ") {
        return Err(Error::VariableNameHasSpaces(command_no));
    }

    let variable = Variable::Static {
        name: name.to_string(),
        value: value.to_string(),
    };

    Ok(Command::Set(variable))
}

fn setsh(arguments: &str, command_no: u64) -> Result<Command, Error> {
    let mut arguments = arguments.splitn(2, ",");

    let name = arguments
        .nth(0)
        .ok_or(Error::NotEnoughArugments(command_no))?
        .trim();

    let shell_script = PathBuf::from(
        arguments
            .nth(0)
            .ok_or(Error::NotEnoughArugments(command_no))?
            .trim(),
    );

    if name.contains(" ") {
        return Err(Error::VariableNameHasSpaces(command_no));
    }

    let variable = Variable::Dynamic {
        name: name.to_string(),
        shell_script,
        value: None,
    };

    Ok(Command::Setsh(variable))
}

fn name(arguments: &str, command_no: u64) -> Result<Command, Error> {
    empty_string_chk(arguments, command_no)?;

    Ok(Command::Name(arguments.trim().to_string()))
}

fn description(arguments: &str, command_no: u64) -> Result<Command, Error> {
    empty_string_chk(arguments, command_no)?;

    Ok(Command::Description(arguments.trim().to_string()))
}

fn version(arguments: &str, command_no: u64) -> Result<Command, Error> {
    empty_string_chk(arguments, command_no)?;

    Ok(Command::Version(arguments.trim().to_string()))
}

fn src(arguments: &str, command_no: u64) -> Result<Command, Error> {
    empty_string_chk(arguments, command_no)?;

    Ok(Command::Src(arguments.trim().to_string()))
}

fn srcl(arguments: &str, command_no: u64) -> Result<Command, Error> {
    empty_string_chk(arguments, command_no)?;

    Ok(Command::Srcl(arguments.trim().to_string()))
}

fn dep(arguments: &str, command_no: u64) -> Result<Command, Error> {
    empty_string_chk(arguments, command_no)?;

    Ok(Command::Dep(arguments.trim().to_string()))
}

fn archive_type(arguments: &str, command_no: u64) -> Result<Command, Error> {
    let archive_type = match arguments.trim() {
        "zip" => ArchiveType::Zip,
        "tar" => ArchiveType::Tar,
        "other" => ArchiveType::Other,
        _ => return Err(Error::UnknownArchiveType(command_no)),
    };

    Ok(Command::ArchiveType(archive_type))
}

fn bin(arguments: &str, command_no: u64) -> Result<Command, Error> {
    empty_string_chk(arguments, command_no)?;

    Ok(Command::Bin(arguments.trim().to_string()))
}

fn build(arguments: &str, command_no: u64) -> Result<Command, Error> {
    empty_string_chk(arguments, command_no)?;

    Ok(Command::Build(arguments.trim().to_string()))
}

fn uninstall(arguments: &str, command_no: u64) -> Result<Command, Error> {
    empty_string_chk(arguments, command_no)?;

    Ok(Command::Uninstall(arguments.trim().to_string()))
}
