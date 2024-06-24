use entity::origin::Model as OriginModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[schema(example = json!({"origin": {"id": 42, "subdomain_id": 1, "value": "https://example.com"}}))]
pub struct GetOriginResponse {
    pub origin: OriginModel,
}
