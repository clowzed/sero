use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[schema(example = json!({"origin": "https://example.com/"}))]
pub struct AddOriginRequest {
    /// Origin to be added
    pub origin: String,
}
