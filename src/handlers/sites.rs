use crate::{
    apperror::SeroError,
    extractors::{
        AuthJWT, Subdomain as SubdomainExtractor, SubdomainModel as SubdomainModelExtractor,
    },
    services::sites::SitesService,
    AppState,
};
use entity::prelude::*;

use sea_orm::prelude::*;

use axum::{
    body::StreamBody,
    extract::{Path, State},
    http::{
        header::{self, HeaderMap},
        StatusCode,
    },
    response::{IntoResponse, Response},
};

use axum::body::Bytes;
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use std::sync::Arc;
use tokio_util::io::ReaderStream;

#[derive(TryFromMultipart)]
pub struct UploadData {
    pub archive: FieldData<Bytes>,
}

#[tracing::instrument(skip(state, archive))]
pub async fn upload(
    State(state): State<Arc<AppState>>,
    AuthJWT(user): AuthJWT,
    SubdomainExtractor(subdomain_name): SubdomainExtractor,
    TypedMultipart(UploadData { archive }): TypedMultipart<UploadData>,
) -> Response {
    match user
        .find_related(SubdomainEntity)
        .all(&state.connection)
        .await
    {
        Ok(subdomains) => {
            if state.config.max_sites_per_user.is_some()
                && subdomains.len() >= state.config.max_sites_per_user.unwrap()
                && !subdomains
                    .iter()
                    .any(|subdomain| subdomain.name == subdomain_name)
            {
                return SeroError::MaxSitesPerUserLimitExceeded.into_response();
            }
        }
        Err(cause) => return SeroError::InternalServerError(Box::new(cause)).into_response(),
    };

    let subdomain = match SitesService::associate(user, &subdomain_name, &state.connection).await {
        Ok(Some(subdomain)) => subdomain,
        Ok(None) => {
            return SeroError::SubdomainIsOwnedByAnotherUser(subdomain_name).into_response()
        }
        Err(cause) => return SeroError::InternalServerError(Box::new(cause)).into_response(),
    };

    match SitesService::upload(&subdomain, archive.contents, &state.connection).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(cause) => return SeroError::InternalServerError(Box::new(cause)).into_response(),
    }
}

#[tracing::instrument(skip(state))]
pub async fn teardown(
    State(state): State<Arc<AppState>>,
    AuthJWT(user): AuthJWT,
    SubdomainModelExtractor(subdomain): SubdomainModelExtractor,
) -> Response {
    if subdomain.owner_id != user.id {
        return SeroError::SubdomainIsOwnedByAnotherUser(subdomain.name).into_response();
    }

    if let Err(cause) = SitesService::teardown(subdomain, &state.connection).await {
        return SeroError::InternalServerError(Box::new(cause)).into_response();
    }
    StatusCode::OK.into_response()
}

#[tracing::instrument()]
pub async fn download(
    AuthJWT(user): AuthJWT,
    SubdomainModelExtractor(subdomain): SubdomainModelExtractor,
) -> Response {
    if !subdomain.owner_id == user.id {
        return SeroError::SubdomainIsOwnedByAnotherUser(subdomain.name).into_response();
    }
    match SitesService::download(&subdomain).await {
        Some(path) => StreamBody::new(ReaderStream::new(
            tokio::fs::File::open(path).await.unwrap(),
        ))
        .into_response(),
        None => SeroError::ArchiveFileWasNotFoundForSubdomain(subdomain.name).into_response(),
    }
}

pub async fn index_redirect(
    State(state): State<Arc<AppState>>,
    SubdomainModelExtractor(subdomain): SubdomainModelExtractor,
) -> Response {
    file(
        State(state),
        SubdomainModelExtractor(subdomain),
        Path(String::from("index.html")),
    )
    .await
}

pub async fn file(
    State(state): State<Arc<AppState>>,
    SubdomainModelExtractor(subdomain): SubdomainModelExtractor,
    Path(path): Path<String>,
) -> Response {
    if !subdomain.enabled {
        return match SitesService::getfile(&subdomain, "503.html".to_owned(), &state.connection)
            .await
        {
            Ok(Some((is_404, file))) if !is_404 => (
                StatusCode::SERVICE_UNAVAILABLE,
                StreamBody::new(ReaderStream::new(
                    tokio::fs::File::open(file).await.unwrap(),
                )),
            )
                .into_response(),
            Err(cause) => SeroError::InternalServerError(Box::new(cause)).into_response(),
            _ => SeroError::SiteDisabled.into_response(),
        };
    }

    match SitesService::getfile(&subdomain, path, &state.connection).await {
        Ok(Some((is_404, file))) => {
            let mut headers = HeaderMap::new();
            if let Some(mime_type) = mime_guess::from_path(&file).first() {
                headers.insert(header::CONTENT_TYPE, mime_type.to_string().parse().unwrap());
            }
            (
                match is_404 {
                    true => StatusCode::NOT_FOUND,
                    false => StatusCode::OK,
                },
                headers,
                StreamBody::new(ReaderStream::new(
                    tokio::fs::File::open(file).await.unwrap(),
                )),
            )
                .into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(cause) => SeroError::InternalServerError(Box::new(cause)).into_response(),
    }
}

#[tracing::instrument(skip(state))]
pub async fn disable(
    State(state): State<Arc<AppState>>,
    AuthJWT(user): AuthJWT,
    SubdomainModelExtractor(subdomain): SubdomainModelExtractor,
) -> Response {
    if subdomain.owner_id != user.id {
        return SeroError::SubdomainIsOwnedByAnotherUser(subdomain.name).into_response();
    }

    if let Err(cause) = SitesService::disable(subdomain, &state.connection).await {
        return SeroError::InternalServerError(Box::new(cause)).into_response();
    }
    StatusCode::OK.into_response()
}

#[tracing::instrument(skip(state))]
pub async fn enable(
    State(state): State<Arc<AppState>>,
    AuthJWT(user): AuthJWT,
    SubdomainModelExtractor(subdomain): SubdomainModelExtractor,
) -> Response {
    if subdomain.owner_id != user.id {
        return SeroError::SubdomainIsOwnedByAnotherUser(subdomain.name).into_response();
    }

    if let Err(cause) = SitesService::enable(subdomain, &state.connection).await {
        return SeroError::InternalServerError(Box::new(cause)).into_response();
    }
    StatusCode::OK.into_response()
}
