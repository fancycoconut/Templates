use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub logging: LoggingSettings,
}

#[derive(Debug, Deserialize)]
pub struct LoggingSettings {
    pub level: String,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        Config::builder()
            .add_source(config::File::with_name("config").format(config::FileFormat::Toml))
            .add_source(config::Environment::default().separator("__").prefix("APP"))
            .build()?
            .try_deserialize()
    }
}
