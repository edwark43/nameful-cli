use color_eyre::eyre::OptionExt;
use serde::Deserialize;
use std::{fs, io::Write};
use xdg::BaseDirectories;

#[derive(Deserialize)]
pub struct Config {
    pub api_key: String,
}

impl Config {
    pub fn init() -> color_eyre::Result<Config> {
        let xdg_dirs = BaseDirectories::with_prefix("nameful-cli");
        let config_path = xdg_dirs.place_config_file("config.toml")?;
        if xdg_dirs.find_config_file(&config_path) == None {
            let mut config_file = fs::File::create(&config_path)?;
            write!(&mut config_file, "api_key = \"\"")?;
        }
        Config::new()
    }
    pub fn new() -> color_eyre::Result<Config> {
        let xdg_dirs = BaseDirectories::with_prefix("nameful-cli");
        let config_path = xdg_dirs
            .find_config_file("config.toml")
            .ok_or_eyre("could not find config toml")?;
        let content = fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&content)?)
    }
}
