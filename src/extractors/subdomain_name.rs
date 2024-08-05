use crate::{state::State, Details};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
pub struct SubdomainName(pub String);

#[derive(thiserror::Error, Debug)]
pub enum SubdomainNameError {
    #[error("x-subdomain header missing")]
    XSubdomainHeaderMissing,
    #[error("x-subdomain header contains bad characters")]
    XSubdomainHeaderContainsBadChars,
}

impl From<SubdomainNameError> for StatusCode {
    fn from(value: SubdomainNameError) -> Self {
        match value {
            SubdomainNameError::XSubdomainHeaderMissing | SubdomainNameError::XSubdomainHeaderContainsBadChars => {
                StatusCode::BAD_REQUEST
            }
        }
    }
}

impl IntoResponse for SubdomainNameError {
    fn into_response(self) -> Response {
        let reason = self.to_string();
        let status_code: StatusCode = self.into();

        tracing::error!(%reason, %status_code, "Error occurred while trying to handle request!");
        (status_code, Json(Details { reason })).into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for SubdomainName
where
    Arc<State>: FromRef<S>,

    S: Send + Sync,
{
    type Rejection = SubdomainNameError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self({
            let header = parts
                .headers
                .get("x-subdomain")
                .ok_or(SubdomainNameError::XSubdomainHeaderMissing)?
                .to_str()
                .map_err(|_| SubdomainNameError::XSubdomainHeaderContainsBadChars)?
                .to_ascii_lowercase();

            match header.is_empty() {
                true => Err(SubdomainNameError::XSubdomainHeaderMissing),
                false => Ok(header),
            }?
        }))
    }
}
