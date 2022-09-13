use core::convert::TryFrom;
use serde::Deserialize;
use std::fs;
use toml::de::Error as TomlError;

#[derive(Debug, Deserialize, Default)]
pub struct MatrixBotConfig {
    pub homeserver: String,
    pub username: String,
    pub password: String,
    pub statedir: Option<String>,
}

impl MatrixBotConfig {
    pub fn new(path: Option<String>) -> Self {
        let config_path = match path {
            Some(a) => a,
            None => match directories::ProjectDirs::from("ca", "ahal", "testbot") {
                Some(dirs) => {
                    let path = dirs.config_dir().join("config.toml");
                    String::from(path.to_str().unwrap())
                }
                None => String::from("config.toml"),
            },
        };
        dbg!(&config_path);

        let contents = fs::read_to_string(&config_path).expect("Error reading config file!");
        MatrixBotConfig::try_from(contents.as_ref()).unwrap()
    }
}

impl TryFrom<&str> for MatrixBotConfig {
    type Error = TomlError;

    fn try_from(buf: &str) -> Result<Self, Self::Error> {
        let config: MatrixBotConfig = toml::from_str(buf)?;
        Ok(config)
    }
}
