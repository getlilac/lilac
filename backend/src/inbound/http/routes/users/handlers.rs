use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    domain::user::{
        models::{ApiKeyId, UserId},
        service::UserService,
    },
    inbound::http::{
        errors::ApiError,
        routes::users::models::{ApiKeyResponse, CreateApiKeyResponse, GetUserHttpResponse},
        AppState,
    },
};

use crate::domain::auth::models::Claims;

#[axum::debug_handler(state = AppState)]
pub async fn get_current_user(
    claims: Claims,
    State(user_service): State<Arc<dyn UserService>>,
) -> Result<Json<GetUserHttpResponse>, ApiError> {
    let user = user_service.get_user_by_id(&claims.sub).await?;
    Ok(Json(user.into()))
}

#[axum::debug_handler(state = AppState)]
pub async fn get_user(
    claims: Claims,
    State(user_service): State<Arc<dyn UserService>>,
    Path(user_id): Path<UserId>,
) -> Result<Json<GetUserHttpResponse>, ApiError> {
    if claims.sub != user_id {
        return Err(ApiError::Forbidden);
    }
    let user = user_service.get_user_by_id(&user_id).await?;
    Ok(Json(user.into()))
}

#[allow(dead_code)]
#[axum::debug_handler(state = AppState)]
pub async fn delete_user(
    claims: Claims,
    State(user_service): State<Arc<dyn UserService>>,
    Path(user_id): Path<UserId>,
) -> Result<(), ApiError> {
    user_service.delete_user(&claims.sub, &user_id).await?;
    Ok(())
}

#[axum::debug_handler(state = AppState)]
pub async fn create_api_key(
    claims: Claims,
    State(user_service): State<Arc<dyn UserService>>,
) -> Result<Json<CreateApiKeyResponse>, ApiError> {
    let new_api_key = user_service.create_api_key(&claims.sub).await?;
    Ok(Json(new_api_key.into()))
}

#[axum::debug_handler(state = AppState)]
pub async fn list_api_keys(
    claims: Claims,
    State(user_service): State<Arc<dyn UserService>>,
) -> Result<Json<Vec<ApiKeyResponse>>, ApiError> {
    let api_keys = user_service.list_api_keys(&claims.sub).await?;
    let response = api_keys.into_iter().map(ApiKeyResponse::from).collect();
    Ok(Json(response))
}

#[axum::debug_handler(state = AppState)]
pub async fn delete_api_key(
    claims: Claims,
    State(user_service): State<Arc<dyn UserService>>,
    Path(key_id): Path<ApiKeyId>,
) -> Result<(), ApiError> {
    user_service.delete_api_key(&claims.sub, &key_id).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            auth::models::Claims,
            user::{
                models::{ApiKey, NewApiKey, User, UserId},
                service::{MockUserService, UserServiceError},
            },
        },
        inbound::http::AppState,
    };
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use chrono::Utc;
    use mockall::predicate::*;
    use secrecy::SecretString;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_current_user() {
        let user = User::new_mock();
        let user_id = user.id;
        let claims = Claims::new_mock(user_id);

        let mut mock_user_service = MockUserService::new();
        mock_user_service
            .expect_get_user_by_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(user.clone()));

        let mut app_state = AppState::new_mock();
        app_state.user_service = Arc::new(mock_user_service);

        let result = get_current_user(claims, State(app_state.user_service)).await;

        assert!(result.is_ok());
        let json_response = result.unwrap();
        assert_eq!(json_response.user_id, user_id);
        assert_eq!(json_response.username, "test_user");
    }

    #[tokio::test]
    async fn test_get_user() {
        let user = User::new_mock();
        let user_id = user.id;
        let claims = Claims::new_mock(user_id);

        let mut mock_user_service = MockUserService::new();
        mock_user_service
            .expect_get_user_by_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(user.clone()));

        let mut app_state = AppState::new_mock();
        app_state.user_service = Arc::new(mock_user_service);

        let result = get_user(claims, State(app_state.user_service), Path(user_id)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_forbidden() {
        let other_user_id = UserId::generate();
        let claims = Claims::new_mock(UserId::generate());
        let app_state = AppState::new_mock();

        let result = get_user(claims, State(app_state.user_service), Path(other_user_id)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.into_response().status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_delete_user() {
        let user_to_delete = User::new_mock();
        let user_id = user_to_delete.id;
        let claims = Claims::new_mock(user_id);

        let mut mock_user_service = MockUserService::new();
        mock_user_service
            .expect_delete_user()
            .with(eq(user_id), eq(user_id))
            .times(1)
            .returning(|_, _| Ok(()));

        let mut app_state = AppState::new_mock();
        app_state.user_service = Arc::new(mock_user_service);

        let result = delete_user(claims, State(app_state.user_service), Path(user_id)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_user_forbidden() {
        let user_to_delete_id = UserId::generate();
        let authenticated_user_id = UserId::generate();
        let claims = Claims::new_mock(authenticated_user_id);

        let mut mock_user_service = MockUserService::new();
        mock_user_service
            .expect_delete_user()
            .with(eq(authenticated_user_id), eq(user_to_delete_id))
            .times(1)
            .returning(|_, _| Err(UserServiceError::InvalidPermissions));

        let mut app_state = AppState::new_mock();
        app_state.user_service = Arc::new(mock_user_service);

        let result = delete_user(
            claims,
            State(app_state.user_service),
            Path(user_to_delete_id),
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.into_response().status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_create_api_key() {
        let user = User::new_mock();
        let user_id = user.id;
        let claims = Claims::new_mock(user_id);

        let new_api_key = NewApiKey {
            id: ApiKeyId::generate(),
            prefix: "lilac_sk_".to_string(),
            key: SecretString::new("test_key".to_string().into()),
            created_at: Utc::now(),
        };

        let mut mock_user_service = MockUserService::new();
        mock_user_service
            .expect_create_api_key()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(new_api_key.clone()));

        let mut app_state = AppState::new_mock();
        app_state.user_service = Arc::new(mock_user_service);

        let result = create_api_key(claims, State(app_state.user_service)).await;

        assert!(result.is_ok());
        let json_response = result.unwrap();
        assert_eq!(json_response.0.prefix, "lilac_sk_");
    }

    #[tokio::test]
    async fn test_list_api_keys() {
        let user = User::new_mock();
        let user_id = user.id;
        let claims = Claims::new_mock(user_id);

        let api_keys = vec![ApiKey {
            id: ApiKeyId::generate(),
            user_id: Some(user_id),
            cluster_id: None,
            prefix: "lilac_sk_".to_string(),
            key_hash: "hash".to_string(),
            created_at: Utc::now(),
            last_used_at: None,
            expires_at: None,
        }];

        let mut mock_user_service = MockUserService::new();
        mock_user_service
            .expect_list_api_keys()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(api_keys.clone()));

        let mut app_state = AppState::new_mock();
        app_state.user_service = Arc::new(mock_user_service);

        let result = list_api_keys(claims, State(app_state.user_service)).await;

        assert!(result.is_ok());
        let json_response = result.unwrap();
        assert_eq!(json_response.0.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_api_key() {
        let user = User::new_mock();
        let user_id = user.id;
        let claims = Claims::new_mock(user_id);
        let key_id = ApiKeyId::generate();

        let mut mock_user_service = MockUserService::new();
        mock_user_service
            .expect_delete_api_key()
            .with(eq(user_id), eq(key_id))
            .times(1)
            .returning(|_, _| Ok(()));

        let mut app_state = AppState::new_mock();
        app_state.user_service = Arc::new(mock_user_service);

        let result = delete_api_key(claims, State(app_state.user_service), Path(key_id)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_api_key_not_found() {
        let user = User::new_mock();
        let user_id = user.id;
        let claims = Claims::new_mock(user_id);
        let key_id = ApiKeyId::generate();

        let mut mock_user_service = MockUserService::new();
        mock_user_service
            .expect_delete_api_key()
            .with(eq(user_id), eq(key_id))
            .times(1)
            .returning(|_, _| Err(UserServiceError::ApiKeyNotFound));

        let mut app_state = AppState::new_mock();
        app_state.user_service = Arc::new(mock_user_service);

        let result = delete_api_key(claims, State(app_state.user_service), Path(key_id)).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.into_response().status(), StatusCode::NOT_FOUND);
    }
}
