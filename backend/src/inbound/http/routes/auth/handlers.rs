use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Json,
};

use crate::{
    config::LilacConfig,
    domain::{auth::models::Token, user::service::UserService},
    inbound::http::{
        errors::ApiError,
        routes::auth::models::{LoginHttpRequest, SignUpHttpRequest, SignUpHttpResponse},
        AppState,
    },
};
use tower_sessions::Session;

pub async fn login_with_username(
    State(app_state): State<AppState>,
    Json(req): Json<LoginHttpRequest>,
) -> Result<Json<Token>, ApiError> {
    let token = app_state
        .auth_service
        .login_with_username(&req.username, &req.password)
        .await?;
    Ok(Json(token))
}

pub async fn logout(session: Session) -> Result<impl IntoResponse, ApiError> {
    session.clear().await;
    Ok(Redirect::to("/"))
}

pub async fn sign_up(
    State(config): State<Arc<LilacConfig>>,
    State(user_service): State<Arc<dyn UserService>>,
    Json(req): Json<SignUpHttpRequest>,
) -> Result<Json<SignUpHttpResponse>, ApiError> {
    if config.disable_sign_up {
        return Err(ApiError::Forbidden);
    } else if let Some(allowed_usernames) = &config.allowed_usernames {
        if !allowed_usernames.contains(&req.username) {
            return Err(ApiError::Forbidden);
        }
    }
    let user = user_service.create_user(&req.into()).await?;
    Ok(Json(SignUpHttpResponse { user_id: user.id }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{extract::State, Json};
    use chrono::Utc;
    use uuid::Uuid;

    use crate::{
        config::LilacConfig,
        domain::user::{models::User, service::MockUserService},
        inbound::http::{
            errors::ApiError,
            routes::auth::{handlers::sign_up, models::SignUpHttpRequest},
        },
    };
    #[tokio::test]
    pub async fn test_sign_up_disables_signup() {
        let config = Arc::new(LilacConfig {
            disable_sign_up: true,
            ..Default::default()
        });
        let user_service = Arc::new(MockUserService::new());
        let res = sign_up(
            State(config.clone()),
            State(user_service.clone()),
            Json(SignUpHttpRequest {
                username: "username".into(),
                first_name: None,
                last_name: None,
                password: "password".into(),
            }),
        )
        .await;

        assert!(res.is_err_and(|e| matches!(e, ApiError::Forbidden)));
    }

    #[tokio::test]
    pub async fn test_sign_up_blocks_invalid_usernames() {
        let mut user_service = MockUserService::new();
        user_service
            .expect_create_user()
            .withf(|req| req.username == "username1" || req.username == "username2")
            .returning(|req| {
                Ok(User {
                    id: Uuid::nil().into(),
                    first_name: req.first_name.clone(),
                    last_name: req.last_name.clone(),
                    username: req.username.clone(),
                    password_hash: Some("mock hash".into()),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
            });
        let config = Arc::new(LilacConfig {
            allowed_usernames: Some(vec!["username1".into(), "username2".into()]),
            ..Default::default()
        });
        let user_service = Arc::new(user_service);

        let res = sign_up(
            State(config.clone()),
            State(user_service.clone()),
            Json(SignUpHttpRequest {
                username: "invalid_username".into(),
                first_name: None,
                last_name: None,
                password: "password".into(),
            }),
        )
        .await;
        assert!(res.is_err_and(|e| matches!(e, ApiError::Forbidden)));

        let res = sign_up(
            State(config.clone()),
            State(user_service.clone()),
            Json(SignUpHttpRequest {
                username: "username1".into(),
                first_name: None,
                last_name: None,
                password: "password".into(),
            }),
        )
        .await;
        assert!(res.is_ok_and(|u| u.0.user_id == Uuid::nil().into()));
    }
}
