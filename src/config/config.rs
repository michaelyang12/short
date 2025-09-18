use std::{env, fmt};
use std::num::ParseIntError;
use dotenvy::dotenv;

#[derive(Debug)]
pub enum ConfigError {
    MissingDatabaseUrl,
    InvalidPort(ParseIntError),
    EnvVar(env::VarError),
}

#[derive(Clone)]
pub struct Configuration {
    pub database_url: String,
    pub port: u16,
}

pub fn configure_app() -> Result<Configuration, ConfigError> {
    //setup logging
    tracing_subscriber::fmt().init();

    tracing::debug!("Configuring application...");
    dotenv().ok();
    tracing::debug!("dotenv enabled");

    let database_url = env::var("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?;
    tracing::debug!("database_url: {}", database_url);

    let port = match env::var("PORT") {
        Ok(val) => val.parse::<u16>().map_err(ConfigError::InvalidPort)?,
        Err(_) => 4000,
    };
    tracing::debug!("port: {}", port);
    Ok(Configuration { database_url, port })
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingDatabaseUrl => {
                tracing::error!("Missing database URL");
                write!(f, "DATABASE_URL must be set (e.g. in .env)")
            }
            ConfigError::InvalidPort(e) => {
                tracing::error!("Invalid port value: {}", e);
                write!(f, "Invalid PORT value: {e}")
            },
            ConfigError::EnvVar(e) => {
                tracing::error!("Env variable error: {}", e);
                write!(f, "Environment variable error: {e}")
            }

        }
    }
}

// Optional: implement std::error::Error so you can use `?` with things expecting `Error`
impl std::error::Error for ConfigError {}