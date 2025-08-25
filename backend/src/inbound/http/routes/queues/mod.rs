pub mod handlers;
pub mod models;

use axum::{
    routing::{get, post},
    Router,
};

use crate::inbound::http::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/queues",
            post(handlers::create_queue).get(handlers::list_queues),
        )
        .route(
            "/queues/{queue_id}",
            get(handlers::get_queue)
                .put(handlers::update_queue)
                .delete(handlers::delete_queue),
        )
        .route("/queues/{queue_id}/jobs", get(handlers::list_queue_jobs))
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::{
            queue::{
                models::{Queue, QueueId},
                service::MockQueueService,
            },
            training_job::models::TrainingJob,
        },
        inbound::http::{
            routes::{
                queues::models::{
                    HttpCreateQueueRequest, HttpQueueResponse, HttpUpdateQueueRequest,
                },
                training_jobs::models::ListTrainingJobsHttpResponse,
            },
            AppState,
        },
    };
    use axum::{
        body::{to_bytes, Body},
        http::{self, Request, StatusCode},
    };
    use mockall::predicate::*;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn setup_test_app(queue_service: MockQueueService) -> axum::Router {
        let mut app_state = AppState::new_mock();
        app_state.queue_service = Arc::new(queue_service);
        crate::inbound::http::routes::queues::routes().with_state(app_state)
    }

    #[tokio::test]
    async fn test_create_queue_route() {
        let request_body = HttpCreateQueueRequest {
            name: "new-test-queue".to_string(),
            priority: 50,
            cluster_targets: vec![],
        };

        let mut mock_queue_service = MockQueueService::new();
        mock_queue_service
            .expect_create_queue()
            .times(1)
            .returning(|req| {
                Ok(Queue {
                    id: QueueId::generate(),
                    name: req.name,
                    priority: req.priority,
                    cluster_targets: req.cluster_targets,
                })
            });

        let app = setup_test_app(mock_queue_service);

        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/queues")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_body: HttpQueueResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_body.name, "new-test-queue");
    }

    #[tokio::test]
    async fn test_list_queues_route() {
        let queues_to_return = vec![Queue::new_mock()];
        let mut mock_queue_service = MockQueueService::new();
        mock_queue_service
            .expect_list_all_queues()
            .times(1)
            .returning(move || Ok(queues_to_return.clone()));

        let app = setup_test_app(mock_queue_service);

        let request = Request::builder()
            .uri("/queues")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_body: Vec<HttpQueueResponse> = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_body.len(), 1);
    }

    #[tokio::test]
    async fn test_get_queue_route() {
        let queue_to_return = Queue::new_mock();
        let queue_id = queue_to_return.id;
        let mut mock_queue_service = MockQueueService::new();
        mock_queue_service
            .expect_get_queue_by_id()
            .with(eq(queue_id))
            .times(1)
            .returning(move |_| Ok(queue_to_return.clone()));

        let app = setup_test_app(mock_queue_service);

        let request = Request::builder()
            .uri(format!("/queues/{}", queue_id))
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_body: HttpQueueResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_body.id, queue_id);
    }

    #[tokio::test]
    async fn test_update_queue_route() {
        let queue_id = QueueId::generate();
        let request_body = HttpUpdateQueueRequest {
            name: "updated-name".to_string(),
            priority: 10,
            cluster_targets: vec![],
        };

        let mut mock_queue_service = MockQueueService::new();
        mock_queue_service
            .expect_update_queue()
            .times(1)
            .returning(move |req| {
                Ok(Queue {
                    id: req.id,
                    name: req.name,
                    priority: req.priority,
                    cluster_targets: req.cluster_targets,
                })
            });

        let app = setup_test_app(mock_queue_service);

        let request = Request::builder()
            .method(http::Method::PUT)
            .uri(format!("/queues/{}", queue_id))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_body: HttpQueueResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(response_body.name, "updated-name");
    }

    #[tokio::test]
    async fn test_delete_queue_route() {
        let queue_id = QueueId::generate();
        let mut mock_queue_service = MockQueueService::new();
        mock_queue_service
            .expect_delete_queue()
            .with(eq(queue_id))
            .times(1)
            .returning(|_| Ok(()));

        let app = setup_test_app(mock_queue_service);

        let request = Request::builder()
            .method(http::Method::DELETE)
            .uri(format!("/queues/{}", queue_id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_queue_jobs_route() {
        let queue_id = QueueId::generate();
        let jobs_to_return = vec![TrainingJob::new_mock()];
        let mut mock_queue_service = MockQueueService::new();
        mock_queue_service
            .expect_list_queues_jobs()
            .with(eq(queue_id))
            .times(1)
            .returning(move |_| Ok(jobs_to_return.clone()));

        let app = setup_test_app(mock_queue_service);

        let request = Request::builder()
            .uri(format!("/queues/{}/jobs", queue_id))
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let _: ListTrainingJobsHttpResponse = serde_json::from_slice(&body).unwrap();
    }
}
