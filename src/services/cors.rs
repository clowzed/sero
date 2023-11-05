use entity::prelude::*;
use sea_orm::{ConnectionTrait, DbErr, ModelTrait};

use sea_orm::TransactionTrait;

pub struct CorsService;

impl CorsService {
    #[tracing::instrument(skip(connection))]
    pub async fn check<T: ConnectionTrait + TransactionTrait>(
        subdomain: Subdomain,
        origin: &str,
        connection: &T,
    ) -> Result<bool, DbErr> {
        Ok(subdomain
            .find_related(CorsEntity)
            .all(connection)
            .await?
            .iter()
            .any(|origin_model| origin_model.matches(origin)))
    }
}
