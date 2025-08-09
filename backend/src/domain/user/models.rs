use std::fmt::Display;

use chrono::{DateTime, Utc};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{domain::cluster::models::ClusterId, identifier};

identifier!(UserId);

#[derive(Clone, Debug)]
pub struct User {
    pub id: UserId,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: String,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CreateUserRequest {
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password: SecretString,
}

// --- API Key Models ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ApiKeyId(Uuid);

impl ApiKeyId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }

    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn inner(&self) -> &Uuid {
        &self.0
    }
}

impl Display for ApiKeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ApiKeyId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// Represents a newly generated API key, including the plaintext secret.
/// This struct is only created once and given to the user. It is not stored.
#[derive(Clone, Debug, Serialize)]
pub struct NewApiKey {
    pub id: ApiKeyId,
    pub prefix: String,
    #[serde(serialize_with = "crate::domain::serialize_secret_string")]
    pub key: SecretString,
    pub created_at: DateTime<Utc>,
}

/// Represents an API key as stored in the database.
/// The actual key is hashed and stored in `key_hash`.
#[derive(Clone, Debug)]
pub struct ApiKey {
    pub id: ApiKeyId,
    pub user_id: Option<UserId>,
    pub cluster_id: Option<ClusterId>,
    pub prefix: String,
    pub key_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
impl User {
    /// Creates a mock User instance.
    pub fn new_mock() -> Self {
        Self {
            id: Uuid::new_v4().into(),
            username: "test_user".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            password_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
