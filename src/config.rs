use regex::Regex;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use crate::command::{Command, Key};
use crate::keycode::KeyCode;


#[derive(Serialize, Deserialize)]
pub struct DeviceConfig {
    /// Config file version. Generally should not be manually changed.
    pub config_version: u32,
    /// Path of the device whose keys will be partialy replaced by the map below.
    pub device_path: PathBuf,

    /// List of keys that will be rewritten by {default,title}_map.
    pub hw_keys: Vec<KeyCode>,
    /// Replacement commands for hw_keys when no title_map entry
    /// matches the current title.
    pub default_map: Vec<Command>,
    /// List of pairs mapping regex expression of window title to list
    /// of new commands.
    pub title_map: Vec<TitleMap>,
}

#[derive(Serialize, Deserialize)]
pub struct TitleMap {
    #[serde(with = "serde_regex")]
    pub title_regex: Regex,
    pub commands: Vec<Command>,
}
