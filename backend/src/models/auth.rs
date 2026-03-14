use serde::Serialize;
use utoipa::ToSchema;

use super::UserResponse;

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthTokens {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    #[schema(example = "dGhpcyBpcyBhIHJlZnJlc2ggdG9rZW4...")]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub tokens: AuthTokens,
}
