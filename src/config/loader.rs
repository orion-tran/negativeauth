use std::{fs, io};

use anyhow::Result;
use tracing::info;

use super::structs::Config;

pub(crate) fn read_config() -> Result<Config> {
    info!("reading config");
    let config_content = match fs::read_to_string("config.toml") {
        Ok(content) => content,
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => create_config()?,
            _ => return Err(e.into()),
        },
    };
    Ok(toml::from_str(&config_content)?)
}

fn create_config() -> Result<String> {
    info!("no config found, writing default config");
    let default_config_content = include_str!("default-config.toml");
    fs::write("config.toml", default_config_content)?;
    Ok(fs::read_to_string("config.toml")?)
}
