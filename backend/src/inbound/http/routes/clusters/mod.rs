use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::inbound::http::AppState;

mod handlers;
use handlers::*;
mod models;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/clusters", post(create_cluster).get(list_clusters))
        .route(
            "/clusters/{cluster_id}",
            get(get_cluster).delete(delete_cluster),
        )
        .route("/clusters/{cluster_id}/info", get(get_cluster_info))
        .route("/clusters/{cluster_id}/nodes", get(list_cluster_nodes))
        .route(
            "/clusters/{cluster_id}/api-keys",
            post(create_api_key_for_cluster).get(list_api_keys),
        )
        .route(
            "/clusters/{cluster_id}/api-keys/{key_id}",
            delete(delete_cluster_api_key),
        )
        .route("/clusters/{cluster_id}/jobs", get(list_cluster_jobs))
        .route("/nodes/{node_id}", get(get_node))
        .route("/node/{node_id}/status", post(cluster_node_heartbeat))
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::{
            auth::models::TokenClaims,
            cluster::{
                models::{
                    Cluster, ClusterDetails, ClusterId, ClusterNode, ClusterSummary, JobInfo,
                    NodeId,
                },
                service::MockClusterService,
            },
            training_job::{
                models::{JobId, TrainingJob, TrainingJobStatus},
                service::MockTrainingJobService,
            },
            user::models::{ApiKey, ApiKeyId, NewApiKey, UserId},
        },
        inbound::http::{
            routes::clusters::models::{
                CreateClusterHttpRequest, HttpClusterNodeHeartbeat, HttpHeartbeatResponse,
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
        cluster_service: MockClusterService,
        auth_service: crate::domain::auth::service::MockAuthService,
        training_job_service: MockTrainingJobService,
    ) -> axum::Router {
        let mut app_state = AppState::new_mock();
        app_state.cluster_service = Arc::new(cluster_service);
        app_state.auth_service = Arc::new(auth_service);
        app_state.training_job_service = Arc::new(training_job_service);
        crate::inbound::http::routes::clusters::router().with_state(app_state)
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

    fn mock_cluster_auth(cluster_id: ClusterId, token: &'static str) -> MockClusterService {
        let mut cluster_service = MockClusterService::new();
        cluster_service
            .expect_authenticate_by_api_key()
            .withf(move |t| t.expose_secret() == token)
            .times(1)
            .returning(move |_| {
                Ok(Cluster {
                    id: cluster_id,
                    ..Default::default()
                })
            });
        cluster_service
    }

    #[tokio::test]
    async fn test_create_api_key_for_cluster_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let cluster_id = ClusterId::generate();
        let api_key_to_return = NewApiKey::default();
        let api_key_for_closure = api_key_to_return.clone();

        let mut mock_cluster_service = MockClusterService::new();
        mock_cluster_service
            .expect_create_api_key_for_cluster()
            .with(eq(cluster_id))
            .times(1)
            .returning(move |_| Ok(api_key_for_closure.clone()));

        let app = setup_test_app(
            mock_cluster_service,
            mock_user_auth(user_id, token),
            MockTrainingJobService::new(),
        );

        let request = Request::builder()
            .method("POST")
            .uri(format!("/clusters/{}/api-keys", cluster_id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_body: NewApiKey = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_body.id, api_key_to_return.id);
    }

    #[tokio::test]
    async fn test_create_cluster_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let request_body = CreateClusterHttpRequest {
            cluster_name: "Test Cluster".to_string(),
            cluster_description: Some("A description".to_string()),
        };

        let mut mock_cluster_service = MockClusterService::new();
        mock_cluster_service
            .expect_create_cluster()
            .times(1)
            .returning(|_| {
                Ok(Cluster {
                    id: ClusterId::generate(),
                    ..Default::default()
                })
            });

        let app = setup_test_app(
            mock_cluster_service,
            mock_user_auth(user_id, token),
            MockTrainingJobService::new(),
        );

        let request = Request::builder()
            .method("POST")
            .uri("/clusters")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_clusters_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let clusters_to_return = vec![ClusterSummary::default()];

        let mut mock_cluster_service = MockClusterService::new();
        mock_cluster_service
            .expect_list_clusters()
            .times(1)
            .returning(move || Ok(clusters_to_return.clone()));

        let app = setup_test_app(
            mock_cluster_service,
            mock_user_auth(user_id, token),
            MockTrainingJobService::new(),
        );

        let request = Request::builder()
            .uri("/clusters")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_cluster_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let cluster = Cluster {
            id: ClusterId::generate(),
            ..Default::default()
        };
        let cluster_id = cluster.id;

        let mut mock_cluster_service = MockClusterService::new();
        mock_cluster_service
            .expect_get_cluster_by_id()
            .with(eq(cluster_id))
            .times(1)
            .returning(move |_| Ok(cluster.clone()));

        let app = setup_test_app(
            mock_cluster_service,
            mock_user_auth(user_id, token),
            MockTrainingJobService::new(),
        );

        let request = Request::builder()
            .uri(format!("/clusters/{}", cluster_id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_cluster_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let cluster_id = ClusterId::generate();

        let mut mock_cluster_service = MockClusterService::new();
        mock_cluster_service
            .expect_delete_cluster()
            .with(eq(cluster_id))
            .times(1)
            .returning(|_| Ok(()));

        let app = setup_test_app(
            mock_cluster_service,
            mock_user_auth(user_id, token),
            MockTrainingJobService::new(),
        );

        let request = Request::builder()
            .method("DELETE")
            .uri(format!("/clusters/{}", cluster_id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_cluster_info_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let cluster_id = ClusterId::generate();
        let cluster_details = ClusterDetails::default();

        let mut mock_cluster_service = MockClusterService::new();
        mock_cluster_service
            .expect_get_cluster_details()
            .with(eq(cluster_id))
            .times(1)
            .returning(move |_| Ok(cluster_details.clone()));

        let app = setup_test_app(
            mock_cluster_service,
            mock_user_auth(user_id, token),
            MockTrainingJobService::new(),
        );

        let request = Request::builder()
            .uri(format!("/clusters/{}/info", cluster_id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_cluster_nodes_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let cluster_id = ClusterId::generate();
        let nodes_to_return = vec![ClusterNode::new_mock()];

        let mut mock_cluster_service = MockClusterService::new();
        mock_cluster_service
            .expect_list_cluster_nodes()
            .with(eq(cluster_id))
            .times(1)
            .returning(move |_| Ok(nodes_to_return.clone()));

        let app = setup_test_app(
            mock_cluster_service,
            mock_user_auth(user_id, token),
            MockTrainingJobService::new(),
        );

        let request = Request::builder()
            .uri(format!("/clusters/{}/nodes", cluster_id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_api_keys_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let cluster_id = ClusterId::generate();
        let keys_to_return = vec![ApiKey {
            cluster_id: Some(cluster_id),
            ..Default::default()
        }];

        let mut mock_cluster_service = MockClusterService::new();
        mock_cluster_service
            .expect_list_api_keys()
            .with(eq(cluster_id))
            .times(1)
            .returning(move |_| Ok(keys_to_return.clone()));

        let app = setup_test_app(
            mock_cluster_service,
            mock_user_auth(user_id, token),
            MockTrainingJobService::new(),
        );

        let request = Request::builder()
            .uri(format!("/clusters/{}/api-keys", cluster_id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_cluster_api_key_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let cluster_id = ClusterId::generate();
        let key_id = ApiKeyId::generate();

        let mut mock_cluster_service = MockClusterService::new();
        mock_cluster_service
            .expect_delete_cluster_api_key()
            .with(eq(cluster_id), eq(key_id))
            .times(1)
            .returning(|_, _| Ok(()));

        let app = setup_test_app(
            mock_cluster_service,
            mock_user_auth(user_id, token),
            MockTrainingJobService::new(),
        );

        let request = Request::builder()
            .method("DELETE")
            .uri(format!("/clusters/{}/api-keys/{}", cluster_id, key_id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_cluster_jobs_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let cluster_id = ClusterId::generate();
        let jobs_to_return = vec![TrainingJob::new_mock()];

        let mut mock_cluster_service = MockClusterService::new();
        mock_cluster_service
            .expect_list_cluster_jobs()
            .with(eq(cluster_id))
            .times(1)
            .returning(move |_| Ok(jobs_to_return.clone()));

        let app = setup_test_app(
            mock_cluster_service,
            mock_user_auth(user_id, token),
            MockTrainingJobService::new(),
        );

        let request = Request::builder()
            .uri(format!("/clusters/{}/jobs", cluster_id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_node_route() {
        let user_id = UserId::generate();
        let token = "user-token";
        let node_id = NodeId::generate();
        let node_to_return = ClusterNode {
            id: node_id,
            ..ClusterNode::new_mock()
        };

        let mut mock_cluster_service = MockClusterService::new();
        mock_cluster_service
            .expect_get_node_by_id()
            .with(eq(node_id))
            .times(1)
            .returning(move |_| Ok(node_to_return.clone()));

        let app = setup_test_app(
            mock_cluster_service,
            mock_user_auth(user_id, token),
            MockTrainingJobService::new(),
        );

        let request = Request::builder()
            .uri(format!("/nodes/{}", node_id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cluster_node_heartbeat_with_assigned_job() {
        let cluster_id = ClusterId::generate();
        let node_id = NodeId::generate();
        let job_id = JobId::generate();
        let cluster_token = "cluster-api-key";
        let heartbeat_body = HttpClusterNodeHeartbeat {
            job_info: Some(JobInfo {
                current_job_id: job_id,
                status: TrainingJobStatus::Running,
            }),
            ..HttpClusterNodeHeartbeat::new_mock()
        };
        let mut mock_cluster_service = mock_cluster_auth(cluster_id, cluster_token);
        mock_cluster_service
            .expect_update_node_status()
            .times(1)
            .returning(move |_| {
                Ok(ClusterNode {
                    assigned_job_id: Some(job_id),
                    ..ClusterNode::new_mock()
                })
            });
        let mut mock_job_service = MockTrainingJobService::new();
        mock_job_service
            .expect_get_training_job_by_id()
            .with(eq(job_id))
            .times(1)
            .returning(move |_| {
                Ok(TrainingJob {
                    id: job_id,
                    ..TrainingJob::new_mock()
                })
            });
        let app = setup_test_app(
            mock_cluster_service,
            crate::domain::auth::service::MockAuthService::new(),
            mock_job_service,
        );
        let request = Request::builder()
            .method("POST")
            .uri(format!("/node/{}/status", node_id))
            .header("Authorization", format!("Bearer {}", cluster_token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&heartbeat_body).unwrap()))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_body: HttpHeartbeatResponse = serde_json::from_slice(&body).unwrap();
        assert!(response_body.assigned_job.is_some());
    }

    #[tokio::test]
    async fn test_cluster_node_heartbeat_without_assigned_job() {
        let cluster_id = ClusterId::generate();
        let node_id = NodeId::generate();
        let cluster_token = "cluster-api-key";
        let heartbeat_body = HttpClusterNodeHeartbeat::new_mock();
        let mut mock_cluster_service = mock_cluster_auth(cluster_id, cluster_token);
        mock_cluster_service
            .expect_update_node_status()
            .times(1)
            .returning(move |_| {
                Ok(ClusterNode {
                    assigned_job_id: None,
                    ..ClusterNode::new_mock()
                })
            });
        let mock_job_service = MockTrainingJobService::new();
        let app = setup_test_app(
            mock_cluster_service,
            crate::domain::auth::service::MockAuthService::new(),
            mock_job_service,
        );
        let request = Request::builder()
            .method("POST")
            .uri(format!("/node/{}/status", node_id))
            .header("Authorization", format!("Bearer {}", cluster_token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&heartbeat_body).unwrap()))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_body: HttpHeartbeatResponse = serde_json::from_slice(&body).unwrap();
        assert!(response_body.assigned_job.is_none());
    }
}
