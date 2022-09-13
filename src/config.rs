use core::convert::TryFrom;
use serde::Deserialize;
use toml::de::Error as TomlError;

#[derive(Debug, Deserialize, Default)]
pub struct MatrixBotConfig<'a> {
    pub homeserver: &'a str,
    pub username: &'a str,
    pub password: &'a str,
    pub statedir: Option<&'a str>,
}

impl<'a> TryFrom<&'a str> for MatrixBotConfig<'a> {
    type Error = TomlError;

    fn try_from(buf: &'a str) -> Result<Self, Self::Error> {
        let config: MatrixBotConfig = toml::from_str(buf)?;
        Ok(config)
    }
}
