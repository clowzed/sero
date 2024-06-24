use super::error::PageError;
use crate::{
    extractors::*,
    site::{parameters::FileSearchParameters, service::Service as SiteService},
    state::State as AppState,
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tokio::fs;
use tokio_util::io::ReaderStream;

pub mod redirect {
    use super::*;

    #[tracing::instrument(skip(state))]
    pub async fn implementation(
        State(state): State<Arc<AppState>>,
        Subdomain(subdomain): Subdomain,
    ) -> Result<impl IntoResponse, PageError> {
        super::implementation(State(state), Subdomain(subdomain), Path(String::from("index.html"))).await
    }
}

#[tracing::instrument(skip(state))]
pub async fn implementation(
    State(state): State<Arc<AppState>>,
    Subdomain(subdomain): Subdomain,
    Path(path): Path<String>,
) -> Result<Response, PageError> {
    let parameters = FileSearchParameters {
        path,
        subdomain_id: subdomain.id,
    };

    let file = SiteService::file(parameters, state.connection()).await?;

    Ok(match file.path() {
        None => StatusCode::from(&file).into_response(),
        Some(path) => (
            StatusCode::from(&file),
            Body::from_stream(ReaderStream::new(fs::File::open(path).await?)),
        )
            .into_response(),
    })
}
