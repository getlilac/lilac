use async_trait::async_trait;
use thiserror::Error;

use super::models::{ApiKey, ApiKeyId, CreateUserRequest, User, UserId};

#[derive(Debug, Error)]
pub enum UserRepositoryError {
    #[error("user with {field} {value} already exists")]
    Duplicate { field: String, value: String },
    #[error("user with id {0} not found")]
    NotFound(String),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create_user(&self, req: &CreateUserRequest) -> Result<User, UserRepositoryError>;
    async fn get_user_by_id(&self, id: &UserId) -> Result<User, UserRepositoryError>;
    async fn get_user_by_username(&self, username: &str) -> Result<User, UserRepositoryError>;
    async fn delete_user(&self, id: &UserId) -> Result<(), UserRepositoryError>;
}

#[derive(Debug, Error)]
pub enum ApiKeyRepositoryError {
    #[error("api key not found")]
    NotFound,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserApiKeyRepository: Send + Sync + 'static {
    async fn create_api_key(&self, key: &ApiKey) -> Result<(), ApiKeyRepositoryError>;
    async fn find_user_by_api_key_hash(
        &self,
        key_hash: &str,
    ) -> Result<User, ApiKeyRepositoryError>;
    async fn list_api_keys_for_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<ApiKey>, ApiKeyRepositoryError>;
    async fn delete_api_key(&self, id: &ApiKeyId) -> Result<(), ApiKeyRepositoryError>;
}
