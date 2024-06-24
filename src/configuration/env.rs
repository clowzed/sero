use std::{fmt::Debug, path::Path};

use super::reader::ConfigurationReader;

#[derive(Debug)]
pub struct EnvConfigurationReader;

impl ConfigurationReader for EnvConfigurationReader {
    type Error = envy::Error;

    #[tracing::instrument]
    fn read<T, P>(path: Option<P>) -> Result<T, Self::Error>
    where
        T: serde::de::DeserializeOwned,
        P: AsRef<Path> + Debug,
    {
        tracing::info!("Reading configuration with EnvConfigurationReader...");
        envy::from_env::<T>()
    }
}
