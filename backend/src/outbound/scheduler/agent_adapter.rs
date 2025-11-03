use std::sync::Arc;

use tracing::debug;

use crate::domain::{
    cluster::{
        models::{ClusterId, NodeId, NodeStatus},
        ports::ClusterRepository,
    },
    training_job::models::{JobId, ResourceRequirements},
};
use thiserror::Error;

use crate::domain::cluster::ports::ClusterRepositoryError;

#[derive(Debug, Error)]
pub enum AgentSchedulerError {
    #[error(transparent)]
    Cluster(#[from] ClusterRepositoryError),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct AgentSchedulerAdapter {
    cluster_repo: Arc<dyn ClusterRepository>,
}

impl AgentSchedulerAdapter {
    pub fn new(cluster_repo: Arc<dyn ClusterRepository>) -> Self {
        Self { cluster_repo }
    }

    pub async fn find_and_allocate_job(
        &self,
        job_id: &JobId,
        cluster_id: &ClusterId,
        requirements: &ResourceRequirements,
    ) -> Result<Option<NodeId>, AgentSchedulerError> {
        let mut nodes = self.cluster_repo.list_cluster_nodes(cluster_id).await?;

        // Filter nodes that are available and meet the resource requirements.
        nodes.retain(|node| {
            let status_ok =
                node.node_status == NodeStatus::Available && node.assigned_job_id.is_none();
            if !status_ok {
                return false;
            }

            let cpu_ok = node.cpu.millicores >= requirements.cpu_millicores;
            let mem_ok = node.memory_mb >= requirements.memory_mb;
            let gpu_ok = match &requirements.gpus {
                None => true,
                Some(req_gpu) => {
                    // Sum all GPU counts across all GPU types on the node
                    let total_gpu_count: i32 = node.gpus.iter().map(|g| g.count).sum();
                    total_gpu_count >= req_gpu.count
                },
            };

            cpu_ok && mem_ok && gpu_ok
        });

        // Sort the suitable nodes by memory in ascending order (best fit).
        nodes.sort_by_key(|node| node.memory_mb);

        let suitable_node = nodes.into_iter().next();

        if let Some(node) = suitable_node {
            debug!("Found suitable node {} for job {}", node.id, job_id);
            self.cluster_repo
                .assign_job_to_node(&node.id, job_id)
                .await?;
            Ok(Some(node.id))
        } else {
            Ok(None)
        }
    }
}
