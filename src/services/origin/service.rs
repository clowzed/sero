use super::error::ServiceError;
use entity::prelude::*;
use sea_orm::{prelude::*, Set, TransactionTrait};

use std::fmt::Debug;

pub struct Service;

impl Service {
    #[tracing::instrument(skip(connection))]
    pub async fn check_if_origin_is_allowed_for<S, O, C>(
        subdomain_name: S,
        origin: O,
        connection: &C,
    ) -> Result<bool, ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
        S: AsRef<str> + Debug,
        O: AsRef<str> + Debug,
    {
        let subdomain = match SubdomainEntity::find()
            .filter(SubdomainColumn::Name.eq(subdomain_name.as_ref()))
            .one(connection)
            .await?
        {
            Some(subdomain) => Ok(subdomain),
            None => Err(ServiceError::SubdomainWasNotFound(subdomain_name.as_ref().to_owned())),
        }?;

        Ok(subdomain
            .find_related(OriginEntity)
            .all(connection)
            .await?
            .iter()
            .any(|origin_model| origin_model.value == "*" || origin_model.value == origin.as_ref()))
    }

    #[tracing::instrument(skip(connection))]
    pub async fn add_origin_for<C, O>(subdomain_id: i64, origin: O, connection: &C) -> Result<OriginModel, ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
        O: AsRef<str> + Debug,
    {
        let origin_to_be_inserted = OriginActiveModel {
            subdomain_id: Set(subdomain_id),
            value: Set(origin.as_ref().to_string()),
            ..Default::default()
        };

        Ok(OriginEntity::insert(origin_to_be_inserted)
            .exec_with_returning(connection)
            .await?)
    }

    #[tracing::instrument(skip(connection))]
    pub async fn delete_origins_for<C>(subdomain_id: i64, connection: &C) -> Result<u64, ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
    {
        let rows_affected = OriginEntity::delete_many()
            .filter(OriginColumn::SubdomainId.eq(subdomain_id))
            .exec(connection)
            .await?
            .rows_affected;
        Ok(rows_affected)
    }

    #[tracing::instrument(skip(connection))]
    pub async fn retrieve_origins_for<C>(subdomain_id: i64, connection: &C) -> Result<Vec<OriginModel>, ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
    {
        Ok(OriginEntity::find()
            .filter(OriginColumn::SubdomainId.eq(subdomain_id))
            .all(connection)
            .await?)
    }

    pub async fn delete_origin_of<C>(subdomain_id: i64, origin_id: i64, connection: &C) -> Result<u64, ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
    {
        let origin = OriginEntity::find_by_id(origin_id)
            .one(connection)
            .await?
            .ok_or(ServiceError::OriginWasNotFound(origin_id))?;

        if origin.subdomain_id != subdomain_id {
            return Err(ServiceError::OriginDoesNotBelongToSubdomain(origin_id, subdomain_id));
        }

        let rows_affected = origin.delete(connection).await?.rows_affected;
        Ok(rows_affected)
    }

    pub async fn fetch_origin_of<C>(
        subdomain_id: i64,
        origin_id: i64,
        connection: &C,
    ) -> Result<OriginModel, ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
    {
        let origin = OriginEntity::find_by_id(origin_id)
            .one(connection)
            .await?
            .ok_or(ServiceError::OriginWasNotFound(origin_id))?;

        if origin.subdomain_id != subdomain_id {
            return Err(ServiceError::OriginDoesNotBelongToSubdomain(origin_id, subdomain_id));
        }

        Ok(origin)
    }
}
