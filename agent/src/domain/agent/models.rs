use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the static hardware resources of a compute node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResources {
    pub cpu: Cpu,
    pub gpus: Vec<Gpu>,
    pub memory_mb: i32,
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
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, strum::EnumString, strum::Display,
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
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, strum::EnumString, strum::Display, strum::EnumIter,
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
    #[serde(rename = "A30")]
    #[strum(serialize = "A30")]
    A30,
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
    #[serde(rename = "L40")]
    #[strum(serialize = "L40")]
    L40,
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


/// The request sent from the agent during a heartbeat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub memory_info: i32,
    pub cpu_info: Cpu,
    pub gpu_info: Vec<Gpu>,
    pub job_info: Option<JobInfo>,
}

/// The response from a heartbeat call, which may include a job to run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub assigned_job: Option<JobDetails>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobInfo {
    pub current_job_id: Uuid,
    pub status: JobStatus,
}

/// The full details of a job, fetched by the agent when assigned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDetails {
    pub id: Uuid,
    pub docker_uri: String,
}

/// The status of a job, reported by the agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Acknowledged,
    Starting,
    Running,
    Succeeded,
    Failed,
}