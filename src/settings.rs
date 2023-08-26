use anyhow::Result;
use config::Config;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub auth: AuthenticationSettings,
}

#[derive(Deserialize)]
pub struct AuthenticationSettings {
    pub email: Option<String>,
    pub password: Option<String>,
}

pub fn load() -> Result<Settings> {
    let settings = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()?;

    let settings = settings.try_deserialize::<Settings>()?;

    Ok(settings)
}
