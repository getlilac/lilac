pub mod models;
pub mod ports;
pub mod service;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::{mock, predicate::eq};

    use super::{
        models::{ApiKey, ApiKeyId, CreateUserRequest, User, UserId},
        ports::{ApiKeyRepositoryError, UserRepositoryError},
        service::{UserService, UserServiceError, UserServiceImpl},
    };

    use chrono::Utc;
    use secrecy::SecretString;

    // A single combined mock that satisfies both UserRepository and UserApiKeyRepository.
    mock! {
        CombinedUserRepo {}

        #[async_trait::async_trait]
        impl super::ports::UserRepository for CombinedUserRepo {
            async fn create_user(
                &self,
                req: &super::models::CreateUserRequest,
            ) -> Result<User, UserRepositoryError>;
            async fn get_user_by_id(&self, id: &UserId) -> Result<User, UserRepositoryError>;
            async fn get_user_by_username(&self, username: &str) -> Result<User, UserRepositoryError>;
            async fn delete_user(&self, id: &UserId) -> Result<(), UserRepositoryError>;
        }

        #[async_trait::async_trait]
        impl super::ports::UserApiKeyRepository for CombinedUserRepo {
            async fn create_api_key(
                &self,
                key: &ApiKey,
            ) -> Result<(), ApiKeyRepositoryError>;
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
    }

    fn make_mock_api_key(id: ApiKeyId, user_id: UserId) -> ApiKey {
        ApiKey {
            id,
            user_id: Some(user_id),
            cluster_id: None,
            prefix: "lilac_sk_abc".to_string(),
            key_hash: "hash".to_string(),
            created_at: Utc::now(),
            last_used_at: None,
            expires_at: None,
        }
    }

    #[tokio::test]
    async fn test_delete_user_self_succeeds() {
        let mut mock_repo = MockCombinedUserRepo::new();
        let user_id = UserId::generate();

        mock_repo
            .expect_delete_user()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Ok(()));

        let service = UserServiceImpl::new(Arc::new(mock_repo));
        let result = service.delete_user(&user_id, &user_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_user_other_user_fails() {
        let mock_repo = MockCombinedUserRepo::new();
        let current_user_id = UserId::generate();
        let target_user_id = UserId::generate();

        let service = UserServiceImpl::new(Arc::new(mock_repo));
        let result = service.delete_user(&current_user_id, &target_user_id).await;

        assert!(matches!(result, Err(UserServiceError::InvalidPermissions)));
    }

    #[tokio::test]
    async fn test_delete_api_key_owned_by_user_succeeds() {
        let mut mock_repo = MockCombinedUserRepo::new();
        let user_id = UserId::generate();
        let key_id = ApiKeyId::generate();
        let key = make_mock_api_key(key_id, user_id);

        mock_repo
            .expect_list_api_keys_for_user()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(vec![key.clone()]));

        mock_repo
            .expect_delete_api_key()
            .with(eq(key_id))
            .times(1)
            .returning(|_| Ok(()));

        let service = UserServiceImpl::new(Arc::new(mock_repo));
        let result = service.delete_api_key(&user_id, &key_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_api_key_not_owned_by_user_fails() {
        let mut mock_repo = MockCombinedUserRepo::new();
        let user_id = UserId::generate();
        let key_id = ApiKeyId::generate();
        // The user has no API keys, so the target key_id is not in their list.
        mock_repo
            .expect_list_api_keys_for_user()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Ok(vec![]));

        let service = UserServiceImpl::new(Arc::new(mock_repo));
        let result = service.delete_api_key(&user_id, &key_id).await;

        assert!(matches!(result, Err(UserServiceError::InvalidPermissions)));
    }

    #[tokio::test]
    async fn test_create_user_delegates_to_repo() {
        let mut mock_repo = MockCombinedUserRepo::new();
        let req = CreateUserRequest {
            username: "alice".to_string(),
            first_name: Some("Alice".to_string()),
            last_name: None,
            password: SecretString::from("secret"),
        };

        mock_repo
            .expect_create_user()
            .times(1)
            .returning(|_| Ok(User::new_mock()));

        let service = UserServiceImpl::new(Arc::new(mock_repo));
        let result = service.create_user(&req).await;

        assert!(result.is_ok());
    }
}
