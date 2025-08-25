use crate::{
    domain::training_job::models::{JobId, TrainingJobStatus},
    identifier,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

identifier!(ClusterId);
identifier!(NodeId);

#[derive(Clone, Debug, Default)]
pub struct Cluster {
    pub id: ClusterId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default)]
pub struct ClusterSummary {
    pub id: ClusterId,
    pub name: String,
    pub description: Option<String>,
    pub total_nodes: i64,
    pub busy_nodes: i64,
    pub total_running_jobs: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CreateClusterRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, strum::EnumString, strum::Display,
)]
pub enum CpuManufacturer {
    #[serde(rename = "Intel")]
    #[strum(serialize = "Intel")]
    Intel,
    #[serde(rename = "AMD")]
    #[strum(serialize = "AMD")]
    Amd,
    #[serde(rename = "AWS")]
    #[strum(serialize = "AWS")]
    Aws,
}

#[derive(Clone, Debug, Serialize, Deserialize, strum::EnumString, strum::Display)]
pub enum Architecture {
    #[serde(rename = "arm64")]
    #[strum(serialize = "arm64")]
    Arm64,
    #[serde(rename = "arm64-mac")]
    #[strum(serialize = "arm64-mac")]
    Arm64Mac,
    #[serde(rename = "i386")]
    #[strum(serialize = "i386")]
    I386,
    #[serde(rename = "x86_64")]
    #[strum(serialize = "x86_64")]
    X86_64,
    #[serde(rename = "x86_64-mac")]
    #[strum(serialize = "x86_64-mac")]
    X86_64Mac,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cpu {
    pub manufacturer: CpuManufacturer,
    pub architecture: Architecture,
    pub millicores: i32,
}

#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, strum::EnumString, strum::Display,
)]
pub enum GpuManufacturer {
    #[serde(rename = "Nvidia")]
    #[strum(serialize = "Nvidia")]
    Nvidia,
    #[serde(rename = "AMD")]
    #[strum(serialize = "AMD")]
    Amd,
    #[serde(rename = "Habana")]
    #[strum(serialize = "Habana")]
    Habana,
}

#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, strum::EnumString, strum::Display,
)]
pub enum GpuModel {
    #[serde(rename = "Radeon Pro V520")]
    #[strum(serialize = "Radeon Pro V520")]
    RadeonProV520,
    #[serde(rename = "Gaudi HL-205")]
    #[strum(serialize = "Gaudi HL-205")]
    GaudiHL205,
    #[serde(rename = "A100")]
    #[strum(serialize = "A100")]
    A100,
    #[serde(rename = "A10G")]
    #[strum(serialize = "A10G")]
    A10G,
    #[serde(rename = "B200")]
    #[strum(serialize = "B200")]
    B200,
    #[serde(rename = "H100")]
    #[strum(serialize = "H100")]
    H100,
    #[serde(rename = "H200")]
    #[strum(serialize = "H200")]
    H200,
    #[serde(rename = "L4")]
    #[strum(serialize = "L4")]
    L4,
    #[serde(rename = "L40S")]
    #[strum(serialize = "L40S")]
    L40S,
    #[serde(rename = "T4")]
    #[strum(serialize = "T4")]
    T4,
    #[serde(rename = "T4g")]
    #[strum(serialize = "T4g")]
    T4g,
    #[serde(rename = "V100")]
    #[strum(serialize = "V100")]
    V100,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gpu {
    pub manufacturer: GpuManufacturer,
    pub model: GpuModel,
    pub count: i32,
    pub memory_mb: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    #[serde(rename = "available")]
    Available,
    #[serde(rename = "busy")]
    Busy,
}

#[derive(Clone, Debug)]
pub struct ClusterNode {
    pub id: NodeId,
    pub cluster_id: ClusterId,
    pub node_status: NodeStatus,
    pub heartbeat_timestamp: DateTime<Utc>,
    pub memory_mb: i32,
    pub cpu: Cpu,
    pub gpu: Option<Gpu>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assigned_job_id: Option<JobId>,
    pub reported_job_id: Option<JobId>,
}

impl ClusterNode {
    pub fn create(
        node_id: NodeId,
        cluster_id: ClusterId,
        memory_mb: i32,
        cpu: Cpu,
        gpu: Option<Gpu>,
    ) -> Self {
        Self {
            id: node_id,
            cluster_id,
            node_status: NodeStatus::Available,
            heartbeat_timestamp: Utc::now(),
            memory_mb,
            cpu,
            gpu,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            assigned_job_id: None,
            reported_job_id: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobInfo {
    pub current_job_id: JobId,
    pub status: TrainingJobStatus,
}

#[derive(Clone, Debug)]
pub struct UpdateNodeStatusRequest {
    pub node_id: NodeId,
    pub cluster_id: ClusterId,
    pub heartbeat_timestamp: DateTime<Utc>,
    pub memory_info: i32,
    pub cpu_info: Cpu,
    pub gpu_info: Option<Gpu>,
    pub job_info: Option<JobInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ClusterMemoryStats {
    pub total_memory_mb: i64,
    pub used_memory_mb: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ClusterCpuStats {
    pub total_millicores: i64,
    pub used_millicores: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ClusterGpuStats {
    pub total_gpus: i64,
    pub used_gpus: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ClusterJobStats {
    pub total_running_jobs: i64,
}

#[derive(Clone, Debug, Default)]
pub struct ClusterDetails {
    pub id: ClusterId,
    pub name: String,
    pub description: Option<String>,
    pub total_nodes: i64,
    pub busy_nodes: i64,
    pub memory_info: ClusterMemoryStats,
    pub cpu_info: ClusterCpuStats,
    pub gpu_info: ClusterGpuStats,
    pub job_info: ClusterJobStats,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
impl Cpu {
    pub fn new_mock() -> Self {
        Self {
            manufacturer: CpuManufacturer::Intel,
            architecture: Architecture::X86_64,
            millicores: 1000,
        }
    }
}

#[cfg(test)]
impl ClusterNode {
    pub fn new_mock() -> Self {
        Self {
            id: NodeId::generate(),
            cluster_id: ClusterId::generate(),
            node_status: NodeStatus::Available,
            heartbeat_timestamp: Utc::now(),
            memory_mb: 0,
            cpu: Cpu::new_mock(),
            gpu: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            assigned_job_id: None,
            reported_job_id: None,
        }
    }
}
