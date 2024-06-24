use crate::Configuration;
use sea_orm::prelude::*;

#[derive(Debug)]
pub struct State {
    connection: DatabaseConnection,
    configuration: Configuration,
}

impl State {
    pub fn new(connection: DatabaseConnection, configuration: Configuration) -> Self {
        Self {
            connection,
            configuration,
        }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }
}
