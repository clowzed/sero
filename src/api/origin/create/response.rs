use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq)]
#[schema(example = json!({"id" : "42", "origin": "https://example.com/"}))]
pub struct AddOriginResponse {
    /// Automatically generated id for new origin
    /// This can be used for further management
    pub id: i64,
    /// This duplicates origin from response payload
    /// to match REST specification
    pub origin: String,
}
