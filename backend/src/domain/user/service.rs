use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};

use super::{
    models::{ApiKey, ApiKeyId, CreateUserRequest, NewApiKey, User, UserId},
    ports::{ApiKeyRepositoryError, UserApiKeyRepository, UserRepository, UserRepositoryError},
};

const API_KEY_PREFIX: &str = "lilac_sk_";
const NANOID_ALPHABET: [char; 62] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9',
];

#[derive(Debug, thiserror::Error)]
pub enum UserServiceError {
    #[error("invalid permissions")]
    InvalidPermissions,
    #[error("user with {field} {value} already exists")]
    UserExists { field: String, value: String },
    #[error("user {0} not found")]
    UserNotFound(String),
    #[error("api key not found")]
    ApiKeyNotFound,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl From<UserRepositoryError> for UserServiceError {
    fn from(error: UserRepositoryError) -> Self {
        match error {
            UserRepositoryError::Duplicate { field, value } => Self::UserExists { field, value },
            UserRepositoryError::NotFound(id) => Self::UserNotFound(id),
            UserRepositoryError::Unknown(error) => Self::Unknown(error),
        }
    }
}

impl From<ApiKeyRepositoryError> for UserServiceError {
    fn from(error: ApiKeyRepositoryError) -> Self {
        match error {
            ApiKeyRepositoryError::NotFound => Self::ApiKeyNotFound,
            ApiKeyRepositoryError::Unknown(error) => Self::Unknown(error),
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserService: Send + Sync {
    async fn create_user(&self, req: &CreateUserRequest) -> Result<User, UserServiceError>;
    async fn get_user_by_id(&self, id: &UserId) -> Result<User, UserServiceError>;
    async fn delete_user(
        &self,
        current_user_id: &UserId,
        target_user_id: &UserId,
    ) -> Result<(), UserServiceError>;

    async fn create_api_key(&self, user_id: &UserId) -> Result<NewApiKey, UserServiceError>;
    async fn list_api_keys(&self, user_id: &UserId) -> Result<Vec<ApiKey>, UserServiceError>;
    async fn delete_api_key(
        &self,
        current_user_id: &UserId,
        key_id: &ApiKeyId,
    ) -> Result<(), UserServiceError>;
    async fn authenticate_by_api_key(&self, key: &SecretString) -> Result<User, UserServiceError>;
}

#[derive(Clone)]
pub struct UserServiceImpl<R: UserRepository + UserApiKeyRepository> {
    repo: Arc<R>,
}

impl<R: UserRepository + UserApiKeyRepository> UserServiceImpl<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R: UserRepository + UserApiKeyRepository> UserService for UserServiceImpl<R> {
    async fn create_user(&self, req: &CreateUserRequest) -> Result<User, UserServiceError> {
        Ok(self.repo.create_user(req).await?)
    }

    async fn get_user_by_id(&self, id: &UserId) -> Result<User, UserServiceError> {
        Ok(self.repo.get_user_by_id(id).await?)
    }

    async fn delete_user(
        &self,
        current_user_id: &UserId,
        target_user_id: &UserId,
    ) -> Result<(), UserServiceError> {
        if current_user_id != target_user_id {
            return Err(UserServiceError::InvalidPermissions);
        }
        Ok(self.repo.delete_user(target_user_id).await?)
    }

    async fn create_api_key(&self, user_id: &UserId) -> Result<NewApiKey, UserServiceError> {
        // Ensure the user exists before creating a key
        self.repo.get_user_by_id(user_id).await?;

        let key_id = ApiKeyId::generate();
        let raw_key = nanoid::nanoid!(32, &NANOID_ALPHABET);
        let secret_key = SecretString::from(raw_key);
        let full_key = format!("{}{}", API_KEY_PREFIX, secret_key.expose_secret());

        let mut hasher = Sha256::new();
        hasher.update(full_key.as_bytes());
        let key_hash = format!("{:x}", hasher.finalize());

        let api_key = ApiKey {
            id: key_id,
            user_id: Some(*user_id),
            cluster_id: None,
            prefix: API_KEY_PREFIX.to_string(),
            key_hash,
            created_at: Utc::now(),
            last_used_at: None,
            expires_at: None,
        };

        self.repo.create_api_key(&api_key).await?;

        Ok(NewApiKey {
            id: key_id,
            prefix: API_KEY_PREFIX.to_string(),
            key: SecretString::from(full_key),
            created_at: api_key.created_at,
        })
    }

    async fn list_api_keys(&self, user_id: &UserId) -> Result<Vec<ApiKey>, UserServiceError> {
        Ok(self.repo.list_api_keys_for_user(user_id).await?)
    }

    async fn delete_api_key(
        &self,
        current_user_id: &UserId,
        key_id: &ApiKeyId,
    ) -> Result<(), UserServiceError> {
        // TODO: Ensure the user has permission to delete the key
        let keys = self.repo.list_api_keys_for_user(current_user_id).await?;
        if !keys.iter().any(|k| k.id == *key_id) {
            return Err(UserServiceError::InvalidPermissions);
        }

        Ok(self.repo.delete_api_key(key_id).await?)
    }

    async fn authenticate_by_api_key(&self, key: &SecretString) -> Result<User, UserServiceError> {
        let mut hasher = Sha256::new();
        hasher.update(key.expose_secret().as_bytes());
        let key_hash = format!("{:x}", hasher.finalize());

        Ok(self.repo.find_user_by_api_key_hash(&key_hash).await?)
    }
}
