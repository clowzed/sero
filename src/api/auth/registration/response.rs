use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(example = json!({"id" : 1293983717}))]
pub struct RegistrationResponse {
    /// Auto generated id of a registered user
    pub id: i64,
}
