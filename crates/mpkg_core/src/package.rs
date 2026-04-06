use std::{collections::HashMap, path::PathBuf, process::ExitStatus};

use fs_extra::dir::CopyOptions;

use crate::{
    cache::Cache,
    directory::{create_runtime_dirs, get_package_dirs, get_packages_dir},
    error::Error,
    lang_primitives::{Command, Context, Dependency, PackageContext},
};

// [package]
pub fn get_all_packages() -> Result<Vec<(String, bool)>, Error> {
    let packages_dir = get_packages_dir()?;

    packages_dir
        .read_dir()?
        .filter(|e| e.as_ref().map(|e| e.path().is_dir()).unwrap_or(false))
        .map(|e| -> Result<(String, bool), Error> {
            let e = e?;

            let name = e
                .file_name()
                .into_string()
                .map_err(|_| Error::OsStrToStrConversionError)?;

            let mut cache_path = e.path();
            cache_path.push("cache.toml");

            Ok((name, cache_path.exists()))
        })
        .collect()
}

pub fn display_package_metadata(package: &str) -> Result<(), Error> {
    let Some(cache) = get_package_cache(package)? else {
        println!("Package: {}\nStatus: NOT BUILT\n", package);
        return Ok(());
    };

    println!(
        "Package: {} @ {}\nDecription: {}\nBinary: {}\nStatus: BUILT\n",
        cache.name,
        cache.version,
        cache.description,
        cache
            .bin
            .unwrap_or(PathBuf::from("Non-binary package"))
            .to_str()
            .ok_or(Error::OsStrToStrConversionError)?,
    );

    Ok(())
}

pub fn list_all_packages() -> Result<(), Error> {
    for package in get_all_packages()? {
        display_package_metadata(&package.0)?;
    }

    Ok(())
}

pub fn check_package_update(package: &str) -> Result<bool, Error> {
    let Some(cache) = get_package_cache(package)? else {
        return Err(Error::PackageNotBuilt(package.to_string()));
    };

    let package_context = get_package_pcontext(package)?;

    for (command_no, (name, (prev_value, cmd))) in cache.dynamic_variables.iter().enumerate() {
        let mut tmp_context = Context::default();

        cmd.evaluate(&mut tmp_context, &package_context, command_no as u64, true)?;

        if tmp_context
            .variables
            .get(name)
            .ok_or(Error::ParserStateCorrupted)?
            .get_value()?
            != *prev_value
        {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn check_all_packages() -> Result<(), Error> {
    for package in get_all_packages()? {
        if !package.1 {
            println!("`{}` - NOT YET BUILT", package.0);
            continue;
        }

        if check_package_update(&package.0)? {
            println!("`{}` - NEW VERSION AVAILABLE", package.0);
        } else {
            println!("`{}` - UP TO DATE", package.0);
        }
    }

    Ok(())
}

pub fn get_package_pcontext(package: &str) -> Result<PackageContext, Error> {
    let (build_dir, install_dir) = get_package_dirs(package)?;

    let mut package_dir = get_packages_dir()?;
    package_dir.push(package);

    Ok(PackageContext {
        build_dir,
        install_dir,
        package_dir,
    })
}

pub fn build_package(package: &str) -> Result<(), Error> {
    let mut package_dir = get_packages_dir()?;
    package_dir.push(package);

    let mut package_spec_path = package_dir.clone();
    package_spec_path.push("build.mpkg");

    // Check if build/install files already exist, if they do, remove them
    let runtime_dirs = get_package_dirs(package)?;

    if runtime_dirs.0.exists() {
        std::fs::remove_dir_all(runtime_dirs.0)?;
    }

    if runtime_dirs.1.exists() {
        std::fs::remove_dir_all(runtime_dirs.1)?;
    }

    // Create build/install directories
    let (build_dir, install_dir) = create_runtime_dirs(package)?;

    // Load the spec file and evaluate it
    let spec_file_contents = String::from_utf8(std::fs::read(package_spec_path)?)?;

    let mut command_no = 0;

    let mut context = Context::default();

    let package_context = PackageContext {
        build_dir: build_dir.clone(),
        install_dir: install_dir.clone(),
        package_dir,
    };

    for command in spec_file_contents.lines() {
        let command_parsed = Command::from_str(command, command_no)?;

        command_parsed.evaluate(&mut context, &package_context, command_no, false)?;

        command_no += 1;
    }

    let cache = validate_context(&context)?;

    let items = build_dir
        .read_dir()?
        .map(|e| -> Result<PathBuf, Error> { Ok(e?.path()) })
        .collect::<Result<Vec<PathBuf>, Error>>()?;

    fs_extra::copy_items(&items, install_dir, &CopyOptions::new().overwrite(true))?;

    write_package_cache(package, &cache)?;

    Ok(())
}

pub fn uninstall_package(package: &str) -> Result<(), Error> {
    let (build_dir, install_dir) = get_package_dirs(package)?;

    let Some(cache) = get_package_cache(package)? else {
        return Err(Error::PackageNotInstalled(package.to_string()));
    };

    if let Some(uninstall_path) = cache.uninstall {
        std::process::Command::new(uninstall_path)
            .current_dir(&install_dir)
            .status()?;
    }

    let mut cache_path = get_package_pcontext(package)?.package_dir;
    cache_path.push("cache.toml");

    std::fs::remove_file(cache_path)?;
    std::fs::remove_dir_all(build_dir)?;
    std::fs::remove_dir_all(install_dir)?;

    Ok(())
}

pub fn validate_context(ctx: &Context) -> Result<Cache, Error> {
    let name = ctx.name.clone().ok_or(Error::PackageDidNotProvideName)?;

    let version = ctx
        .version
        .clone()
        .ok_or(Error::PackageDidNotProvideVersion)?;

    let description = ctx.description.clone().unwrap_or(String::from("N/A"));

    let bin = ctx.bin.clone();

    let uninstall = ctx.uninstall.clone();

    let dynamic_variables = ctx
        .variables
        .iter()
        .filter(|v| v.1.is_dynamic())
        .map(
            |(name, variable)| -> Result<(String, (String, Command)), Error> {
                Ok((
                    name.clone(),
                    (variable.get_value()?, Command::Setsh(variable.clone())),
                ))
            },
        )
        .collect::<Result<HashMap<String, (String, Command)>, Error>>()?;

    let dependencies = ctx
        .dependencies
        .iter()
        .map(|dep| dep.1.clone())
        .collect::<Vec<Dependency>>();

    Ok(Cache {
        bin,
        name,
        description,
        version,
        uninstall,
        dynamic_variables,
        dependencies,
    })
}

pub fn get_package_cache(package: &str) -> Result<Option<Cache>, Error> {
    let mut package_dir = get_packages_dir()?;
    package_dir.push(package);

    let mut cache_path = package_dir.clone();
    cache_path.push("cache.toml");

    if !cache_path.exists() {
        return Ok(None);
    }

    let cache_contents = std::fs::read(cache_path)?;

    let cache: Cache = toml::from_slice(&cache_contents)?;

    Ok(Some(cache))
}

pub fn write_package_cache(package: &str, cache: &Cache) -> Result<(), Error> {
    let mut package_dir = get_packages_dir()?;
    package_dir.push(package);

    let mut cache_path = package_dir.clone();
    cache_path.push("cache.toml");

    let cache_serialised = toml::to_string_pretty(cache)?;

    std::fs::write(cache_path, &cache_serialised)?;

    Ok(())
}

pub fn launch_package_bin(package: &str) -> Result<ExitStatus, Error> {
    let cache = get_package_cache(package)?.ok_or(Error::PackageNotBuilt(package.to_string()))?;

    let Some(bin) = cache.bin else {
        return Err(Error::NonBinaryPackage(package.to_string()));
    };

    let (_, install_dir) = get_package_dirs(package)?;

    let environment = cache
        .dependencies
        .iter()
        .map(|dep| -> Result<(String, String), Error> {
            Ok((dep.get_dep_var_name(), dep.get_dep_install_dir()?))
        })
        .collect::<Result<Vec<(String, String)>, Error>>()?;

    std::process::Command::new(bin)
        .current_dir(install_dir)
        .envs(environment)
        .status()
        .map_err(|e| e.into())
}
