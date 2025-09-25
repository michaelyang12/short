use dotenvy::dotenv;
use std::num::ParseIntError;
use std::{env, fmt};

// maybe use 'thiserror' crate in the future
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

    tracing::info!("Configuring application...");
    dotenv().ok();
    tracing::info!("dotenv enabled");

    let database_url = env::var("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?;
    tracing::info!("database_url: {}", database_url);

    let port = match env::var("PORT") {
        Ok(val) => val.parse::<u16>().map_err(ConfigError::InvalidPort)?,
        Err(_) => 3000,
    };
    tracing::info!("port: {}", port);
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

impl std::error::Error for ConfigError {}