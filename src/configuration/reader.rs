use std::{fmt::Debug, path::Path};

pub trait ConfigurationReader {
    type Error;

    fn read<T, P>(path: Option<P>) -> Result<T, Self::Error>
    where
        T: serde::de::DeserializeOwned,
        P: AsRef<Path> + Debug;
}
