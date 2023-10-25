#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub database_url: String,
    pub max_sites_per_user: Option<usize>,
    pub max_users: Option<u64>,
    pub max_body_limit_size: Option<usize>,
    pub jwt_secret: Option<String>,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        let mut config: Self =
            envy::from_env().expect("Failed to read config from environment variables!");
        if config.jwt_secret.is_none() {
            config.jwt_secret = Some(uuid::Uuid::new_v4().to_string())
        }
        config
    }
}
