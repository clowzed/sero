use entity::prelude::OriginModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[schema(example = json!({"origins": [{"id": 42, "subdomain_id": 1, "value": "https://example.com"}]}))]
pub struct ListOriginsResponse {
    /// List of retrieved origins
    pub origins: Vec<OriginModel>,
}
