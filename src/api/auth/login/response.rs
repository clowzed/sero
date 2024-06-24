use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// The JWT token generated for authentication purposes.
#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[schema(example = json!({"token": "ferwfwerfwer.fwerfwerfwerfwer.fwerfewfr"}))]
pub struct LoginResponse {
    /// Token in JWT format
    pub token: String,
}
