use std::sync::Arc;

use async_trait::async_trait;
use password_auth::{verify_password, VerifyError};
use secrecy::{ExposeSecret, SecretString};

use super::{
    models::{AuthUser, Token, TokenClaims},
    ports::TokenManager,
};
use crate::domain::user::{
    models::User,
    ports::{UserRepository, UserRepositoryError},
};

#[derive(Debug, thiserror::Error)]
pub enum AuthServiceError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("user not found")]
    UserNotFound,
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl From<UserRepositoryError> for AuthServiceError {
    fn from(err: UserRepositoryError) -> Self {
        match err {
            UserRepositoryError::NotFound(_) => Self::UserNotFound,
            UserRepositoryError::Unknown(err) => Self::Unknown(err),
            e => Self::Unknown(e.into()),
        }
    }
}

impl From<VerifyError> for AuthServiceError {
    fn from(err: VerifyError) -> Self {
        match err {
            VerifyError::Parse(parse_error) => {
                tracing::error!(error = ?parse_error, "failed to parse password hash");
                AuthServiceError::Unknown(parse_error.into())
            }
            VerifyError::PasswordInvalid => AuthServiceError::InvalidCredentials,
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthService: Send + Sync {
    async fn login_with_username(
        &self,
        username: &str,
        password: &SecretString,
    ) -> Result<Token, AuthServiceError>;
    fn validate_token(&self, token: &str) -> Result<TokenClaims, AuthServiceError>;
}

pub struct AuthServiceImpl {
    user_repo: Arc<dyn UserRepository>,
    token_manager: Arc<dyn TokenManager>,
}

impl AuthServiceImpl {
    pub fn new(user_repo: Arc<dyn UserRepository>, token_manager: Arc<dyn TokenManager>) -> Self {
        Self {
            user_repo,
            token_manager,
        }
    }

    async fn verify_password(
        &self,
        password: &SecretString,
        user: &User,
    ) -> Result<(), AuthServiceError> {
        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or(AuthServiceError::InvalidCredentials)?;
        verify_password(password.expose_secret(), password_hash)?;

        Ok(())
    }
}

#[async_trait]
impl AuthService for AuthServiceImpl {
    async fn login_with_username(
        &self,
        username: &str,
        password: &SecretString,
    ) -> Result<Token, AuthServiceError> {
        let user = self.user_repo.get_user_by_username(username).await?;
        self.verify_password(password, &user).await?;

        let auth_user = AuthUser {
            id: user.id,
            username: user.username.clone(),
        };

        let token_str = self
            .token_manager
            .create_token(&auth_user)
            .map_err(AuthServiceError::Unknown)?;
        Ok(Token::new(token_str))
    }

    fn validate_token(&self, token: &str) -> Result<TokenClaims, AuthServiceError> {
        Ok(self.token_manager.validate_token(token)?)
    }
}
