use crate::{
    apperror::SeroError,
    extractors::{AuthJWT, SubdomainModel},
    AppState,
};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Form,
};
use entity::prelude::*;
use sea_orm::{prelude::*, Set};
use std::sync::Arc;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct OriginForm {
    pub origin: String,
}

#[tracing::instrument(skip(state))]
pub async fn add_origin(
    State(state): State<Arc<AppState>>,
    SubdomainModel(subdomain_model): SubdomainModel,
    AuthJWT(user): AuthJWT,
    Form(origin_form): Form<OriginForm>,
) -> Response {
    if subdomain_model.owner_id != user.id {
        return SeroError::SubdomainIsOwnedByAnotherUser(subdomain_model.name).into_response();
    }

    let active_cors_origin = ActiveCors {
        origin: Set(origin_form.origin),
        subdomain_id: Set(subdomain_model.id),
        ..Default::default()
    };

    match CorsEntity::insert(active_cors_origin)
        .exec(&state.connection)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(cause) => SeroError::InternalServerError(Box::new(cause)).into_response(),
    }
}

#[tracing::instrument(skip(state))]
pub async fn clear_all(
    State(state): State<Arc<AppState>>,
    SubdomainModel(subdomain_model): SubdomainModel,
    AuthJWT(user): AuthJWT,
) -> Response {
    if subdomain_model.owner_id != user.id {
        return SeroError::SubdomainIsOwnedByAnotherUser(subdomain_model.name).into_response();
    }

    match CorsEntity::delete_many()
        .filter(CorsColumn::SubdomainId.eq(subdomain_model.id))
        .exec(&state.connection)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(cause) => SeroError::InternalServerError(Box::new(cause)).into_response(),
    }
}
