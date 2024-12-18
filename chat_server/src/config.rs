use std::{env, fs::File, path::PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
    pub base_dir: PathBuf, // PathBuf is ownership, Path is not,  it is like String vs &str
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthConfig {
    pub sk: String,
    pub pk: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // read from /etc/config/chat.yml, or ./chat.yml, or from env CHAT_CONFIG
        let ret = match (
            File::open("./chat.yml"),
            File::open("/etc/config/chat.yml"),
            env::var("CHAT_CONFIG"),
        ) {
            (Ok(file), _, _) => serde_yaml::from_reader(file),
            (_, Ok(file), _) => serde_yaml::from_reader(file),
            (_, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!("Config not found"),
        };

        Ok(ret?)
    }
}
