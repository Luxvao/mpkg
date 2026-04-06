use std::path::PathBuf;

use crate::error::Error;

pub fn init() -> Result<(), Error> {
    let user = std::env::var("USER")?;

    let mpkg_dir = PathBuf::from(format!("/home/{user}/.mpkg"));
    let mut mpkg_packages_dir = mpkg_dir.clone();
    mpkg_packages_dir.push("packages/");

    if mpkg_packages_dir.exists() && mpkg_packages_dir.is_dir() {
        return Ok(());
    }

    std::fs::create_dir_all(mpkg_packages_dir)?;

    Ok(())
}

pub fn get_packages_dir() -> Result<PathBuf, Error> {
    let user = std::env::var("USER")?;

    Ok(PathBuf::from(format!("/home/{user}/.mpkg/packages/")))
}

pub fn get_package_dirs(package: &str) -> Result<(PathBuf, PathBuf), Error> {
    let mut package_dir = get_packages_dir()?;
    package_dir.push(package);

    if !package_dir.exists() {
        return Err(Error::PackageDoesNotExist(package.to_string()));
    }

    let mut build_dir = package_dir.clone();
    let mut install_dir = package_dir.clone();

    build_dir.push("build/");
    install_dir.push("install/");

    Ok((build_dir, install_dir))
}

pub fn create_runtime_dirs(package: &str) -> Result<(PathBuf, PathBuf), Error> {
    let dirs = get_package_dirs(package)?;

    std::fs::create_dir_all(dirs.0.clone())?;
    std::fs::create_dir_all(dirs.1.clone())?;

    Ok(dirs)
}
