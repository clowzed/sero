use super::{error::ServiceError, parameters::*};
use entity::prelude::*;
use futures::Stream;
use sea_orm::{prelude::*, ConnectionTrait, ModelTrait, Set, StreamTrait, TransactionTrait};
use std::{fmt::Debug, path::PathBuf};

pub struct Service;

impl Service {
    #[tracing::instrument(skip(connection))]
    pub async fn teardown<C, P>(parameters: P, connection: &C) -> Result<u64, ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
        P: Into<ActionParameters> + Debug,
    {
        let provided_parameters = parameters.into();

        let subdomain = SubdomainEntity::find_by_id(provided_parameters.subdomain_id)
            .one(connection)
            .await?
            .ok_or(ServiceError::SubdomainWasNotFound)?;

        let files_to_be_removed = FileEntity::update_many()
            .filter(FileColumn::SubdomainId.eq(subdomain.id))
            .col_expr(FileColumn::Obsolete, Expr::value(true))
            .exec(connection)
            .await?
            .rows_affected;

        subdomain.delete(connection).await?;

        Ok(files_to_be_removed)
    }

    #[tracing::instrument(skip(connection))]
    pub async fn archive<C, P>(parameters: P, connection: &C) -> Result<String, ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
        P: Into<ActionParameters> + Debug,
    {
        let provided_parameters = parameters.into();

        SubdomainEntity::find_by_id(provided_parameters.subdomain_id)
            .one(connection)
            .await?
            .ok_or(ServiceError::SubdomainWasNotFound)
            .and_then(|subdomain| subdomain.archive_path.ok_or(ServiceError::ArchiveNotFound))
    }

    #[tracing::instrument(skip(connection))]
    pub async fn file<C, P, S>(parameters: P, connection: &C) -> Result<SiteFile, ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
        P: Into<FileSearchParameters<S>> + Debug,
        S: AsRef<str>,
    {
        let provided_parameters = parameters.into();

        let subdomain = match SubdomainEntity::find_by_id(provided_parameters.subdomain_id)
            .one(connection)
            .await?
        {
            Some(subdomain) => Ok(subdomain),
            None => Err(ServiceError::SubdomainWasNotFound),
        }?;

        let parsed_path = PathBuf::from(provided_parameters.path.as_ref());

        let path = match parsed_path.extension() {
            Some(_) => provided_parameters.path.as_ref().to_owned(),
            None => parsed_path
                .with_extension("html")
                .to_str()
                .unwrap_or_default()
                .to_owned(),
        };

        let files_filter = match subdomain.enabled {
            true => FileColumn::UserPath.eq(path),
            false => FileColumn::UserPath.eq("503.html"),
        };

        match subdomain
            .find_related(FileEntity)
            .filter(FileColumn::Obsolete.eq(false))
            .filter(files_filter)
            .one(connection)
            .await?
        {
            Some(file) => Ok(match subdomain.enabled {
                true => SiteFile::Found(file.real_path.into()),
                false => SiteFile::Disabled(Some(file.real_path.into())),
            }),
            None => match subdomain.enabled {
                true => match subdomain
                    .find_related(FileEntity)
                    .filter(FileColumn::Obsolete.eq(false))
                    .filter(FileColumn::UserPath.eq("404.html"))
                    .one(connection)
                    .await?
                {
                    Some(file) => Ok(SiteFile::NotFound(Some(file.real_path.into()))),
                    None => Ok(SiteFile::NotFound(None)),
                },
                false => Ok(SiteFile::Disabled(None)),
            },
        }
    }

    #[tracing::instrument(skip(connection))]
    pub async fn enable<C, P>(parameters: P, connection: &C) -> Result<(), ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
        P: Into<ActionParameters> + Debug,
    {
        let provided_parameters = parameters.into();

        let subdomain = match SubdomainEntity::find_by_id(provided_parameters.subdomain_id)
            .one(connection)
            .await?
        {
            Some(subdomain) => Ok(subdomain),
            None => Err(ServiceError::SubdomainWasNotFound),
        }?;

        let mut active_subdomain: SubdomainActiveModel = subdomain.into();

        active_subdomain.enabled = Set(true);
        active_subdomain.update(connection).await?;

        Ok(())
    }

    #[tracing::instrument(skip(connection))]
    pub async fn disable<C, P>(parameters: P, connection: &C) -> Result<(), ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
        P: Into<ActionParameters> + Debug,
    {
        let provided_parameters = parameters.into();

        let subdomain = match SubdomainEntity::find_by_id(provided_parameters.subdomain_id)
            .one(connection)
            .await?
        {
            Some(subdomain) => Ok(subdomain),
            None => Err(ServiceError::SubdomainWasNotFound),
        }?;

        let mut active_subdomain: SubdomainActiveModel = subdomain.into();

        active_subdomain.enabled = Set(false);
        active_subdomain.update(connection).await?;

        Ok(())
    }

    #[tracing::instrument(skip(connection))]
    pub async fn grant_possession<C, P, A>(parameters: P, connection: &C) -> Result<SubdomainModel, ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
        P: Into<AssociateParameters<A>> + Debug,
        A: AsRef<str>,
    {
        let provided_parameters = parameters.into();

        match SubdomainEntity::find()
            .filter(SubdomainColumn::Name.eq(provided_parameters.subdomain_name.as_ref()))
            .one(connection)
            .await?
        {
            Some(subdomain) if subdomain.owner_id == provided_parameters.user_id => Ok(subdomain),
            Some(_) => Err(ServiceError::SubdomainIsOwnedByAnotherUser),
            None => {
                let subdomain_to_be_inserted = entity::subdomain::ActiveModel {
                    owner_id: Set(provided_parameters.user_id),
                    name: Set(provided_parameters.subdomain_name.as_ref().to_owned()),
                    ..Default::default()
                };

                Ok(SubdomainEntity::insert(subdomain_to_be_inserted)
                    .exec_with_returning(connection)
                    .await?)
            }
        }
    }

    pub async fn obsolete<C>(connection: &C) -> Result<impl Stream<Item = Result<FileModel, DbErr>> + '_, ServiceError>
    where
        C: ConnectionTrait + StreamTrait,
    {
        Ok(FileEntity::find()
            .filter(FileColumn::Obsolete.eq(true))
            .stream(connection)
            .await?)
    }
}
