use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::lang_primitives::{Command, Dependency};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cache {
    pub bin: Option<PathBuf>,
    pub name: String,
    pub description: String,
    pub version: String,
    pub uninstall: Option<PathBuf>,
    // HashMap<name, (previous, Command (only Sets variant permitted))>
    pub dynamic_variables: HashMap<String, (String, Command)>,
    pub dependencies: Vec<Dependency>,
}
