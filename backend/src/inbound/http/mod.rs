pub mod errors;
pub mod routes;

use axum::{extract::FromRef, Router};
use http::{HeaderName, Request};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tower_sessions::{service::PrivateCookie, SessionManagerLayer};
use tracing::Level;
use uuid::Uuid;

use crate::{
    config::LilacConfig,
    domain::{
        auth::service::AuthService, cluster::service::ClusterService, queue::service::QueueService,
        training_job::service::TrainingJobService, user::service::UserService,
    },
    inbound::http::routes::{clusters, queues, training_jobs},
    outbound::persistence::postgres::session_repository::PostgresSessionStore,
};

use self::routes::{auth, users};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<LilacConfig>,
    pub cluster_service: Arc<dyn ClusterService>,
    pub user_service: Arc<dyn UserService>,
    pub auth_service: Arc<dyn AuthService>,
    pub training_job_service: Arc<dyn TrainingJobService>,
    pub queue_service: Arc<dyn QueueService>,
}

impl FromRef<AppState> for Arc<LilacConfig> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for Arc<dyn ClusterService> {
    fn from_ref(state: &AppState) -> Self {
        state.cluster_service.clone()
    }
}

impl FromRef<AppState> for Arc<dyn UserService> {
    fn from_ref(state: &AppState) -> Self {
        state.user_service.clone()
    }
}

impl FromRef<AppState> for Arc<dyn AuthService> {
    fn from_ref(state: &AppState) -> Self {
        state.auth_service.clone()
    }
}

impl FromRef<AppState> for Arc<dyn TrainingJobService> {
    fn from_ref(state: &AppState) -> Self {
        state.training_job_service.clone()
    }
}

impl FromRef<AppState> for Arc<dyn QueueService> {
    fn from_ref(state: &AppState) -> Self {
        state.queue_service.clone()
    }
}

pub struct HttpServer {
    app: Router,
    listener: TcpListener,
    address: String,
    port: u16,
}

static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

#[derive(Clone, Default)]
struct UuidRequestId;

impl MakeRequestId for UuidRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string().parse().unwrap();

        Some(RequestId::new(request_id))
    }
}

impl HttpServer {
    pub async fn new(
        app_state: AppState,
        session_layer: SessionManagerLayer<PostgresSessionStore, PrivateCookie>,
        port: u16,
    ) -> anyhow::Result<Self> {
        let app: Router = Router::new()
            .merge(users::router())
            .merge(auth::router())
            .merge(clusters::router())
            .merge(training_jobs::training_jobs_router())
            .merge(queues::routes())
            .layer(
                ServiceBuilder::new()
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                            .on_request(DefaultOnRequest::new().level(Level::DEBUG))
                            .on_response(DefaultOnResponse::new().level(Level::DEBUG))
                            .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
                    )
                    .layer(SetRequestIdLayer::new(X_REQUEST_ID.clone(), UuidRequestId))
                    .layer(PropagateRequestIdLayer::new(X_REQUEST_ID.clone()))
                    .layer(session_layer),
            )
            .layer(CorsLayer::very_permissive())
            .with_state(app_state);

        let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
        Ok(Self {
            app,
            listener,
            address: "0.0.0.0".into(),
            port,
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!("starting server on {}:{}", self.address, self.port);
        axum::serve(self.listener, self.app).await?;
        Ok(())
    }
}

#[cfg(test)]
impl AppState {
    /// Creates a new mock AppState with the provided configuration.
    pub fn new_mock_with_config(config: LilacConfig) -> Self {
        use crate::domain::{
            auth::service::MockAuthService, cluster::service::MockClusterService,
            queue::service::MockQueueService, training_job::service::MockTrainingJobService,
            user::service::MockUserService,
        };

        Self {
            config: Arc::new(config),
            cluster_service: Arc::new(MockClusterService::new()),
            user_service: Arc::new(MockUserService::new()),
            auth_service: Arc::new(MockAuthService::new()),
            training_job_service: Arc::new(MockTrainingJobService::new()),
            queue_service: Arc::new(MockQueueService::new()),
        }
    }

    /// Creates a new mock AppState with default configuration.
    pub fn new_mock() -> Self {
        Self::new_mock_with_config(LilacConfig::default())
    }
}
