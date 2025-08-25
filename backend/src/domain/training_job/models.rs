use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{cluster::models::NodeId, queue::models::QueueId},
    identifier,
};

identifier!(JobId);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TrainingJobStatus {
    Queued,
    Starting,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

/// Describes a specific requirement for a GPU.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuRequirement {
    /// The number of GPUs required.
    pub count: i32,
    /// The specific model of the GPU (e.g., "A100", "V100", "RTX4090").
    /// If None, any GPU model is acceptable.
    pub model: Option<String>,
    /// The minimum required memory for each GPU in gigabytes (e.g., 40, 80).
    /// If None, any GPU memory size is acceptable.
    pub memory_gb: Option<i32>,
}

/// Represents the computational resources required for a training job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_millicores: i32,
    pub memory_mb: i32,
    /// GPU requirements for the job. If None, the job does not require GPUs.
    pub gpus: Option<GpuRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingJob {
    pub id: JobId,
    pub name: String,
    // rename to docker uri
    pub definition: String,
    pub status: TrainingJobStatus,
    pub node_id: Option<NodeId>,
    pub queue_id: Option<QueueId>,
    pub resource_requirements: ResourceRequirements,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default, Deserialize, PartialEq, Clone)]
pub struct GetTrainingJobsFilters {
    pub id: Option<JobId>,
    pub name: Option<String>,
    pub status: Option<TrainingJobStatus>,
}

#[cfg(test)]
impl TrainingJob {
    pub fn new_mock() -> Self {
        Self {
            id: JobId::generate(),
            name: "Default Job".to_string(),
            definition: "default-uri".to_string(),
            status: TrainingJobStatus::Queued,
            node_id: None,
            queue_id: None,
            resource_requirements: ResourceRequirements {
                cpu_millicores: 0,
                memory_mb: 0,
                gpus: None,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}
