use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::PathBuf};
pub mod env;
pub mod reader;
pub use env::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Configuration {
    database_url: String,
    max_sites_per_user: Option<u64>,
    max_users: Option<u64>,
    max_body_limit_size: Option<usize>,
    jwt_secret: String,
    port: u16,
    jwt_ttl_seconds: i64,
    sqlx_logging: bool,
    upload_folder: PathBuf,
    clean_obsolete_interval: Option<u64>,
}

impl Debug for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Configuration")
            .field("database_url", &"***")
            .field("max_sites_per_user", &self.max_sites_per_user)
            .field("max_users", &self.max_users)
            .field("max_body_limit_size", &self.max_body_limit_size)
            .field("jwt_secret", &"***")
            .field("port", &self.port)
            .field("jwt_ttl_seconds", &self.jwt_ttl_seconds)
            .field("sqlx_logging", &self.sqlx_logging)
            .field("upload_folder", &self.upload_folder)
            .field("clean_obsolete_interval", &self.clean_obsolete_interval)
            .finish()
    }
}

impl Configuration {
    pub fn database_url(&self) -> &str {
        self.database_url.as_ref()
    }

    pub fn max_sites_per_user(&self) -> Option<u64> {
        self.max_sites_per_user
    }

    pub fn max_users(&self) -> Option<u64> {
        self.max_users
    }

    pub fn max_body_limit_size(&self) -> Option<usize> {
        self.max_body_limit_size
    }

    pub fn jwt_secret(&self) -> &str {
        self.jwt_secret.as_ref()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn jwt_ttl_seconds(&self) -> i64 {
        self.jwt_ttl_seconds
    }

    pub fn sqlx_logging(&self) -> bool {
        self.sqlx_logging
    }

    pub fn upload_folder(&self) -> &PathBuf {
        &self.upload_folder
    }

    pub fn clean_obsolete_interval(&self) -> Option<u64> {
        self.clean_obsolete_interval
    }
}
