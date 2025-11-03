use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use secrecy::SecretString;

use crate::{
    domain::{
        auth::models::Claims,
        cluster::{
            models::{ClusterId, NodeId, UpdateNodeStatusRequest},
            service::ClusterService,
        },
        training_job::service::TrainingJobService,
        user::models::{ApiKeyId, NewApiKey},
    },
    inbound::http::{
        errors::ApiError,
        routes::clusters::models::{
            CreateClusterHttpRequest, CreateClusterHttpResponse, GetClusterDetailsHttpResponse,
            GetClusterHttpResponse, HttpApiKey, HttpClusterNode, HttpClusterNodeHeartbeat,
            HttpHeartbeatResponse, HttpJobDetails, ListClusterJobsHttpResponse,
            ListClusterNodesHttpResponse, ListClustersHttpResponse,
        },
    },
};

use crate::inbound::http::AppState;

#[axum::debug_handler(state = AppState)]
pub async fn create_cluster(
    _claims: Claims,
    State(cluster_service): State<Arc<dyn ClusterService>>,
    Json(req): Json<CreateClusterHttpRequest>,
) -> Result<Json<CreateClusterHttpResponse>, ApiError> {
    let cluster = cluster_service.create_cluster(&req.into()).await?;
    Ok(Json(CreateClusterHttpResponse {
        cluster_id: cluster.id,
    }))
}

#[axum::debug_handler(state = AppState)]
pub async fn get_cluster(
    _claims: Claims,
    State(cluster_service): State<Arc<dyn ClusterService>>,
    Path(cluster_id): Path<ClusterId>,
) -> Result<Json<GetClusterHttpResponse>, ApiError> {
    let cluster = cluster_service.get_cluster_by_id(&cluster_id).await?;
    Ok(Json(cluster.into()))
}

#[axum::debug_handler(state = AppState)]
pub async fn get_cluster_info(
    _claims: Claims,
    State(cluster_service): State<Arc<dyn ClusterService>>,
    Path(cluster_id): Path<ClusterId>,
) -> Result<Json<GetClusterDetailsHttpResponse>, ApiError> {
    let cluster = cluster_service.get_cluster_details(&cluster_id).await?;
    Ok(Json(cluster.into()))
}

#[axum::debug_handler(state = AppState)]
pub async fn list_clusters(
    _claims: Claims,
    State(cluster_service): State<Arc<dyn ClusterService>>,
) -> Result<Json<ListClustersHttpResponse>, ApiError> {
    let clusters = cluster_service.list_clusters().await?;
    Ok(Json(clusters.into()))
}

#[axum::debug_handler(state = AppState)]
pub async fn delete_cluster(
    _claims: Claims,
    State(cluster_service): State<Arc<dyn ClusterService>>,
    Path(cluster_id): Path<ClusterId>,
) -> Result<(), ApiError> {
    cluster_service.delete_cluster(&cluster_id).await?;
    Ok(())
}

use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

#[axum::debug_handler(state = AppState)]
pub async fn create_api_key_for_cluster(
    _claims: Claims,
    State(cluster_service): State<Arc<dyn ClusterService>>,
    Path(cluster_id): Path<ClusterId>,
) -> Result<Json<NewApiKey>, ApiError> {
    let new_api_key = cluster_service
        .create_api_key_for_cluster(&cluster_id)
        .await?;
    Ok(Json(new_api_key))
}

#[axum::debug_handler(state = AppState)]
pub async fn delete_cluster_api_key(
    _claims: Claims,
    State(cluster_service): State<Arc<dyn ClusterService>>,
    Path((cluster_id, key_id)): Path<(ClusterId, ApiKeyId)>,
) -> Result<(), ApiError> {
    cluster_service
        .delete_cluster_api_key(&cluster_id, &key_id)
        .await?;
    Ok(())
}

#[axum::debug_handler(state = AppState)]
pub async fn list_api_keys(
    _claims: Claims,
    State(cluster_service): State<Arc<dyn ClusterService>>,
    Path(cluster_id): Path<ClusterId>,
) -> Result<Json<Vec<HttpApiKey>>, ApiError> {
    let api_keys = cluster_service.list_api_keys(&cluster_id).await?;
    let http_api_keys = api_keys.into_iter().map(HttpApiKey::from).collect();
    Ok(Json(http_api_keys))
}

#[axum::debug_handler(state = AppState)]
pub async fn cluster_node_heartbeat(
    Path(node_id): Path<NodeId>,
    State(cluster_service): State<Arc<dyn ClusterService>>,
    State(training_job_service): State<Arc<dyn TrainingJobService>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(req): Json<HttpClusterNodeHeartbeat>,
) -> Result<Json<HttpHeartbeatResponse>, ApiError> {
    let cluster = cluster_service
        .authenticate_by_api_key(&SecretString::from(auth.token().to_string()))
        .await?;

    let node = cluster_service
        .update_node_status(UpdateNodeStatusRequest {
            node_id,
            cluster_id: cluster.id,
            heartbeat_timestamp: Utc::now(),
            memory_info: req.memory_info,
            cpu_info: req.cpu_info,
            gpu_info: req.gpu_info,
            job_info: req.job_info,
        })
        .await?;

    // tracing::info!(
    //     node_id = %node.id,
    //     cluster_id = %node.cluster_id,
    //     node_status = ?node.node_status,
    //     memory_mb = node.memory_mb,
    //     cpu_millicores = node.cpu.millicores,
    //     gpus = ?node.gpus,
    //     assigned_job_id = ?node.assigned_job_id,
    //     reported_job_id = ?node.reported_job_id,
    //     "Node heartbeat received and updated"
    // );

    let assigned_job = if let Some(job_id) = node.assigned_job_id {
        let job_details = training_job_service.get_training_job_by_id(&job_id).await?;
        Some(HttpJobDetails::from(job_details))
    } else {
        None
    };

    Ok(Json(HttpHeartbeatResponse { assigned_job }))
}

#[axum::debug_handler(state = AppState)]
pub async fn get_node(
    _claims: Claims,
    State(cluster_service): State<Arc<dyn ClusterService>>,
    Path(node_id): Path<NodeId>,
) -> Result<Json<HttpClusterNode>, ApiError> {
    let node = cluster_service.get_node_by_id(&node_id).await?;
    Ok(Json(node.into()))
}

#[axum::debug_handler(state = AppState)]
pub async fn list_cluster_nodes(
    _claims: Claims,
    State(cluster_service): State<Arc<dyn ClusterService>>,
    Path(cluster_id): Path<ClusterId>,
) -> Result<Json<ListClusterNodesHttpResponse>, ApiError> {
    let cluster_nodes = cluster_service.list_cluster_nodes(&cluster_id).await?;
    Ok(Json(cluster_nodes.into()))
}

#[axum::debug_handler(state = AppState)]
pub async fn list_cluster_jobs(
    _claims: Claims,
    State(cluster_service): State<Arc<dyn ClusterService>>,
    Path(cluster_id): Path<ClusterId>,
) -> Result<Json<ListClusterJobsHttpResponse>, ApiError> {
    let jobs = cluster_service.list_cluster_jobs(&cluster_id).await?;
    Ok(Json(jobs.into()))
}
