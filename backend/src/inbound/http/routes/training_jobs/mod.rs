use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::inbound::http::AppState;

use self::handlers::{
    cancel_training_job, create_training_job, get_training_job, list_training_jobs, post_logs,
    update_training_job_status,
};

pub mod handlers;
pub mod models;

pub fn training_jobs_router() -> Router<AppState> {
    Router::new()
        .route("/training_jobs", post(create_training_job))
        .route("/training_jobs", get(list_training_jobs))
        .route("/training_jobs/{job_id}", get(get_training_job))
        .route(
            "/training_jobs/{job_id}/status",
            patch(update_training_job_status),
        )
        .route("/training_jobs/{job_id}/logs", post(post_logs))
        .route("/training_jobs/{job_id}/cancel", post(cancel_training_job))
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::{
            auth::models::TokenClaims,
            training_job::{
                models::{JobId, TrainingJob, TrainingJobStatus},
                service::MockTrainingJobService,
            },
            user::{models::UserId, service::MockUserService},
        },
        inbound::http::{
            routes::training_jobs::models::{
                CreateTrainingJobRequest, HttpTrainingJob, ListTrainingJobsHttpResponse,
                UpdateTrainingJobStatusRequest,
            },
            AppState,
        },
    };
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use mockall::predicate::*;
    use secrecy::ExposeSecret;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn setup_test_app(
        training_job_service: MockTrainingJobService,
        user_service: MockUserService,
        auth_service: crate::domain::auth::service::MockAuthService,
    ) -> axum::Router {
        let mut app_state = AppState::new_mock();
        app_state.training_job_service = Arc::new(training_job_service);
        app_state.user_service = Arc::new(user_service);
        app_state.auth_service = Arc::new(auth_service);
        crate::inbound::http::routes::training_jobs::training_jobs_router().with_state(app_state)
    }

    fn mock_user_auth(
        user_id: UserId,
        token: &'static str,
    ) -> crate::domain::auth::service::MockAuthService {
        let token_claims = TokenClaims::new_mock(user_id);
        let mut auth_service = crate::domain::auth::service::MockAuthService::new();
        auth_service
            .expect_validate_token()
            .with(eq(token))
            .returning(move |_| Ok(token_claims.clone()));
        auth_service
    }

    #[tokio::test]
    async fn test_create_training_job_route() {
        let api_key = "cluster-api-key";
        let request_body = CreateTrainingJobRequest {
            name: "test-job".to_string(),
            definition: "test-uri".to_string(),
            queue_id: Default::default(),
            resource_requirements: serde_json::Value::Null,
        };

        let mut mock_user_service = MockUserService::new();
        mock_user_service
            .expect_authenticate_by_api_key()
            .withf(move |secret| secret.expose_secret() == api_key)
            .times(1)
            .returning(|_| Ok(Default::default()));

        let mut mock_job_service = MockTrainingJobService::new();
        mock_job_service
            .expect_create()
            .times(1)
            .returning(|_| Ok(TrainingJob::new_mock()));

        let app = setup_test_app(mock_job_service, mock_user_service, Default::default());

        let request = Request::builder()
            .method("POST")
            .uri("/training_jobs")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_list_training_jobs_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let jobs_to_return = vec![TrainingJob::new_mock()];

        let mut mock_job_service = MockTrainingJobService::new();
        mock_job_service
            .expect_get_training_jobs()
            .times(1)
            .returning(move |_| Ok(jobs_to_return.clone()));

        let app = setup_test_app(
            mock_job_service,
            Default::default(),
            mock_user_auth(user_id, token),
        );

        let request = Request::builder()
            .uri("/training_jobs")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let _: ListTrainingJobsHttpResponse = serde_json::from_slice(&body).unwrap();
    }

    #[tokio::test]
    async fn test_get_training_job_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let job_to_return = TrainingJob::new_mock();
        let job_id = job_to_return.id;

        let mut mock_job_service = MockTrainingJobService::new();
        mock_job_service
            .expect_get_training_job_by_id()
            .with(eq(job_id))
            .times(1)
            .returning(move |_| Ok(job_to_return.clone()));

        let app = setup_test_app(
            mock_job_service,
            Default::default(),
            mock_user_auth(user_id, token),
        );

        let request = Request::builder()
            .uri(format!("/training_jobs/{}", job_id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_body: HttpTrainingJob = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_body.job_id, job_id);
    }

    #[tokio::test]
    async fn test_update_training_job_status_route() {
        let job_id = JobId::generate();
        let request_body = UpdateTrainingJobStatusRequest {
            status: TrainingJobStatus::Running,
        };

        let mut mock_job_service = MockTrainingJobService::new();
        mock_job_service
            .expect_update_status()
            .with(eq(job_id), eq(TrainingJobStatus::Running))
            .times(1)
            .returning(|_, _| Ok(()));

        let app = setup_test_app(mock_job_service, Default::default(), Default::default());

        let request = Request::builder()
            .method("PATCH")
            .uri(format!("/training_jobs/{}/status", job_id))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_post_logs_route() {
        let job_id = JobId::generate();

        let mock_job_service = MockTrainingJobService::new();
        let app = setup_test_app(mock_job_service, Default::default(), Default::default());

        let request = Request::builder()
            .method("POST")
            .uri(format!("/training_jobs/{}/logs", job_id))
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"logs": "some log content"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cancel_training_job_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let job_id = JobId::generate();

        let mut mock_job_service = MockTrainingJobService::new();
        mock_job_service
            .expect_cancel()
            .with(eq(job_id))
            .times(1)
            .returning(|_| Ok(()));

        let app = setup_test_app(
            mock_job_service,
            Default::default(),
            mock_user_auth(user_id, token),
        );

        let request = Request::builder()
            .method("POST")
            .uri(format!("/training_jobs/{}/cancel", job_id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
