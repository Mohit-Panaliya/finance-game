use serde::{Deserialize, Serialize};

use crate::models::users::UserResponse;

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub token: String,
}

impl LoginResponse {
    #[must_use]
    pub fn new(user: UserResponse, token: &str) -> Self {
        Self {
            user,
            token: token.to_owned(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrentResponse {
    pub user: UserResponse,
}

impl CurrentResponse {
    #[must_use]
    pub fn new(user: UserResponse) -> Self {
        Self { user }
    }
}
