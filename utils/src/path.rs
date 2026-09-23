//! convenient access to system paths

use std::{ffi::OsString, path::PathBuf};

pub fn home() -> Option<PathBuf> {
    std::env::home_dir()
}

pub fn config() -> Option<PathBuf> {
    dir("XDG_CONFIG_HOME", ".config")
}

pub fn data() -> Option<PathBuf> {
    dir("XDG_DATA_HOME", ".local/share")
}

fn dir(env: &'static str, fallback: &'static str) -> Option<PathBuf> {
    std::env::var_os(env)
        .and_then(is_absolute_path)
        .or_else(|| home().map(|home| home.join(fallback)))
}

fn is_absolute_path(path: OsString) -> Option<PathBuf> {
    let path = PathBuf::from(path);
    match path.is_absolute() {
        true => Some(path),
        false => None,
    }
}
