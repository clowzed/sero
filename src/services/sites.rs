use entity::prelude::*;
use sea_orm::prelude::*;
use sea_orm::Set;
use sea_orm::{ConnectionTrait, DbErr, ModelTrait};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

use sea_orm::TransactionTrait;

use super::archive::ArchiveService;

use super::archive::ArchiveServiceError;

pub struct SitesService;

#[derive(thiserror::Error, Debug)]
pub enum SiteServiceError {
    #[error(transparent)]
    FsError(#[from] std::io::Error),

    #[error(transparent)]
    DbErr(#[from] DbErr),

    #[error(transparent)]
    ArchiveError(#[from] ArchiveServiceError),
}

impl SitesService {
    #[tracing::instrument(skip(connection))]
    pub async fn teardown<T: ConnectionTrait + TransactionTrait>(
        subdomain: Subdomain,
        connection: &T,
    ) -> Result<(), DbErr> {
        if let Some(archive) = &subdomain.archive_path {
            if let Err(cause) = tokio::fs::remove_file(archive).await {
                tracing::warn!(%cause, "Could not remove archive related to subdomain!")
            }
        }

        match subdomain.find_related(FileEntity).all(connection).await {
            Ok(files) => {
                for file in files.iter() {
                    if let Err(cause) = tokio::fs::remove_file(&file.real_path).await {
                        tracing::warn!(%cause, "Could not remove file related to subdomain with path: {}!", file.real_path);
                    }
                }
            }
            Err(cause) => tracing::warn!(%cause, "Could not find file related to subdomain!"),
        };

        subdomain.delete(connection).await?;

        Ok(())
    }

    #[tracing::instrument]
    pub async fn download(subdomain: &Subdomain) -> Option<PathBuf> {
        if let Some(ref path) = subdomain.archive_path {
            let path_as_pathbuf = PathBuf::from(path);

            return match tokio::fs::metadata(&path_as_pathbuf).await {
                Ok(_) => Some(path_as_pathbuf),
                Err(_) => None,
            };
        }

        None
    }

    #[tracing::instrument(skip(connection, contents))]
    pub async fn upload<T: ConnectionTrait + TransactionTrait>(
        subdomain: &Subdomain,
        contents: bytes::Bytes,
        connection: &T,
    ) -> Result<(), SiteServiceError> {
        if let Some(ref old_archive) = subdomain.archive_path {
            if let Err(cause) = tokio::fs::remove_file(old_archive).await {
                tracing::warn!(%cause, "Failed to remove old archive!");
            }
        }

        let new_archive_path = format!("{}.zip", uuid::Uuid::new_v4());

        let mut new_archive_file = match tokio::fs::File::create(&new_archive_path).await {
            Ok(file) => file,
            Err(cause) => {
                return Err(cause.into());
            }
        };

        match new_archive_file.write_all(&contents).await {
            Ok(()) => {
                let mut active: ActiveSubdomain = subdomain.clone().into();
                active.archive_path = Set(Some(new_archive_path));

                if let Err(cause) = active.update(connection).await {
                    return Err(cause.into());
                }
            }
            Err(cause) => {
                return Err(cause.into());
            }
        }

        FileEntity::delete_many()
            .filter(FileColumn::SubdomainId.eq(subdomain.id))
            .exec(connection)
            .await?;

        let files = match ArchiveService::process(contents).await {
            Ok(files) => files,
            Err(cause) => {
                return Err(cause.into());
            }
        };

        let transaction = connection.begin().await?;

        let models = files
            .iter()
            .map(|file| ActiveFile {
                subdomain_id: Set(subdomain.id),
                user_path: Set(file.user_path.to_string_lossy().to_string()),
                real_path: Set(file.real_path.to_string_lossy().to_string()),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        FileEntity::insert_many(models).exec(&transaction).await?;
        transaction.commit().await?;

        Ok(())
    }

    #[tracing::instrument(skip(connection))]
    pub async fn getfile<T: ConnectionTrait + TransactionTrait>(
        subdomain: &Subdomain,
        path: String,
        connection: &T,
    ) -> Result<Option<(bool, std::path::PathBuf)>, SiteServiceError> {
        let files = subdomain
            .find_related(FileEntity)
            .filter(FileColumn::UserPath.is_in([&path, "404.html"]))
            .all(connection)
            .await?;

        let file = match files.len() {
            0 => None,
            1 => Some(&files[0]),
            2 => Some(
                files
                    .iter()
                    .find(|file| !file.user_path.eq("404.html"))
                    .unwrap(),
            ),
            _ => unreachable!(),
        };

        match file {
            Some(file) => match tokio::fs::metadata(&file.real_path).await {
                Ok(_) => Ok(Some((
                    file.user_path == "404.html",
                    std::path::PathBuf::from(&file.real_path),
                ))),
                Err(cause) => Err(cause.into()),
            },
            None => Ok(None),
        }
    }

    pub async fn enable<T: ConnectionTrait + TransactionTrait>(
        subdomain: Subdomain,
        connection: &T,
    ) -> Result<(), DbErr> {
        let mut active_subdomain: ActiveSubdomain = subdomain.into();
        active_subdomain.enabled = Set(true);
        active_subdomain.update(connection).await?;
        Ok(())
    }

    pub async fn disable<T: ConnectionTrait + TransactionTrait>(
        subdomain: Subdomain,
        connection: &T,
    ) -> Result<(), DbErr> {
        let mut active_subdomain: ActiveSubdomain = subdomain.into();
        active_subdomain.enabled = Set(false);
        active_subdomain.update(connection).await?;
        Ok(())
    }

    #[tracing::instrument(skip(connection))]
    pub async fn associate<T: ConnectionTrait + TransactionTrait>(
        user: User,
        subdomain_name: &str,
        connection: &T,
    ) -> Result<Option<Subdomain>, DbErr> {
        match SubdomainEntity::find()
            .filter(SubdomainColumn::Name.eq(subdomain_name))
            .one(connection)
            .await?
        {
            Some(subdomain) => {
                if subdomain.owner_id == user.id {
                    Ok(Some(subdomain))
                } else {
                    Ok(None)
                }
            }
            None => {
                let new_subdomain = entity::subdomain::ActiveModel {
                    owner_id: Set(user.id),
                    name: Set(subdomain_name.to_string()),
                    ..Default::default()
                };

                let new_subdomain = SubdomainEntity::insert(new_subdomain.clone())
                    .exec_with_returning(connection)
                    .await?;

                Ok(Some(new_subdomain))
            }
        }
    }
}
