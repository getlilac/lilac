use secrecy::ExposeSecret;
use std::sync::Arc;
use tower_sessions::{cookie::Key, Expiry, SessionManagerLayer};

use server::{
    config::{LilacConfig, LogFormat},
    domain::{
        auth::service::AuthServiceImpl, cluster::service::ClusterServiceImpl,
        queue::service::QueueServiceImpl, scheduler::service::SchedulerService,
        training_job::service::TrainingJobServiceImpl, user::service::UserServiceImpl,
    },
    inbound::http::{AppState, HttpServer},
    outbound::{
        jwt::JwtManager,
        persistence::postgres::{
            cluster_repository::PostgresClusterRepository,
            queue_repository::PostgresQueueRepository, session_repository::PostgresSessionStore,
            training_job_repository::PostgresTrainingJobRepository,
            user_repository::PostgresUserRepository,
        },
        scheduler::agent_adapter::AgentSchedulerAdapter,
    },
};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 0. Install the default crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install the default crypto provider");

    // 1. Load config
    let config = Arc::new(LilacConfig::new().expect("could not parse provided config"));

    // 2. Initialize tracing
    let trace_subscriber = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()));
    match config.log_format {
        LogFormat::Pretty => trace_subscriber
            .with(tracing_subscriber::fmt::layer().pretty())
            .try_init()
            .expect("no tracing subscriber to already be installed"),
        LogFormat::Json => trace_subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .try_init()
            .expect("no tracing subscriber to already be installed"),
    };

    // 3. Construct outbound adapters
    let db_pool = PgPoolOptions::new()
        .connect(config.database_url.expose_secret())
        .await?;
    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("migrations to run");
    let cluster_repo = Arc::new(PostgresClusterRepository::new(db_pool.clone()));
    let user_repo = Arc::new(PostgresUserRepository::new(db_pool.clone()));
    let jwt_manager = Arc::new(JwtManager::new(config.secret_key.expose_secret()));
    let training_job_repo = Arc::new(PostgresTrainingJobRepository::new(db_pool.clone()));
    let queue_repo = Arc::new(PostgresQueueRepository::new(db_pool.clone()));

    // 3. Construct domain services
    let cluster_service = Arc::new(ClusterServiceImpl::new(
        cluster_repo.clone(),
        training_job_repo.clone(),
    ));
    let user_service = Arc::new(UserServiceImpl::new(user_repo.clone()));
    let session_store = PostgresSessionStore::new(db_pool.clone());
    session_store.migrate().await?;
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_private(Key::from(config.secret_key.expose_secret().as_bytes()))
        .with_expiry(Expiry::OnInactivity(time::Duration::minutes(30)));

    let auth_service = Arc::new(AuthServiceImpl::new(user_repo.clone(), jwt_manager));
    let training_job_service = Arc::new(TrainingJobServiceImpl::new(
        training_job_repo.clone(),
        cluster_repo.clone(),
    ));
    let queue_service = Arc::new(QueueServiceImpl::new(
        queue_repo.clone(),
        training_job_repo.clone(),
    ));

    // 4. Construct Scheduler
    let agent_adapter = Arc::new(AgentSchedulerAdapter::new(cluster_repo.clone()));
    let scheduler_service = Arc::new(SchedulerService::new(
        training_job_repo.clone(),
        queue_repo.clone(),
        cluster_repo.clone(),
        agent_adapter,
    ));

    // 5. Spawn background tasks
    let scheduler_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = scheduler_service.run_cycle().await {
                tracing::error!("Scheduler cycle failed: {}", e);
            }
        }
    });

    // 6. Construct and run inbound adapter (HTTP server)
    let app_state = AppState {
        config: config.clone(),
        cluster_service,
        user_service,
        auth_service,
        training_job_service,
        queue_service,
    };
    let http_server = HttpServer::new(app_state, session_layer, config.http_port).await?;

    // Run the server and wait for it and the scheduler to complete
    tokio::select! {
        _ = http_server.run() => {},
        _ = scheduler_handle => {},
    }

    Ok(())
}
