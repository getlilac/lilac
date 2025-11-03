use crate::domain::{
    cluster::models::{
        Architecture, Cluster, ClusterCpuStats, ClusterDetails, ClusterGpuStats, ClusterJobStats,
        ClusterMemoryStats, ClusterNode, ClusterSummary, Cpu, CpuManufacturer, Gpu,
        GpuManufacturer, GpuModel, NodeStatus,
    },
    training_job::models::{TrainingJob, TrainingJobStatus},
    user::models::ApiKey,
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

//  Cluster Repository Records

#[derive(sqlx::FromRow)]
pub struct ClusterRecord {
    pub cluster_id: uuid::Uuid,
    pub cluster_name: String,
    pub cluster_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ClusterRecord> for Cluster {
    fn from(record: ClusterRecord) -> Self {
        Self {
            id: record.cluster_id.into(),
            name: record.cluster_name,
            description: record.cluster_description,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct ClusterSummaryRecord {
    pub cluster_id: uuid::Uuid,
    pub cluster_name: String,
    pub cluster_description: Option<String>,
    pub total_nodes: i64,
    pub busy_nodes: i64,
    pub total_running_jobs: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ClusterSummaryRecord> for ClusterSummary {
    fn from(record: ClusterSummaryRecord) -> Self {
        Self {
            id: record.cluster_id.into(),
            name: record.cluster_name,
            description: record.cluster_description,
            total_nodes: record.total_nodes,
            busy_nodes: record.busy_nodes,
            total_running_jobs: record.total_running_jobs,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(type_name = "node_status", rename_all = "snake_case")]
pub enum NodeStatusRecord {
    Available,
    Busy,
}

impl From<NodeStatus> for NodeStatusRecord {
    fn from(value: NodeStatus) -> Self {
        match value {
            NodeStatus::Available => NodeStatusRecord::Available,
            NodeStatus::Busy => NodeStatusRecord::Busy,
        }
    }
}

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(type_name = "cpu_manufacturer", rename_all = "PascalCase")]
pub enum CpuManufacturerRecord {
    Intel,
    Amd,
    Aws,
}

impl From<CpuManufacturer> for CpuManufacturerRecord {
    fn from(value: CpuManufacturer) -> Self {
        match value {
            CpuManufacturer::Intel => Self::Intel,
            CpuManufacturer::Amd => Self::Amd,
            CpuManufacturer::Aws => Self::Aws,
        }
    }
}

impl From<CpuManufacturerRecord> for CpuManufacturer {
    fn from(value: CpuManufacturerRecord) -> Self {
        match value {
            CpuManufacturerRecord::Intel => Self::Intel,
            CpuManufacturerRecord::Amd => Self::Amd,
            CpuManufacturerRecord::Aws => Self::Aws,
        }
    }
}

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(type_name = "architecture")]
pub enum ArchitectureRecord {
    #[sqlx(rename = "arm64")]
    Arm64,
    #[sqlx(rename = "arm64-mac")]
    Arm64Mac,
    #[sqlx(rename = "i386")]
    I386,
    #[sqlx(rename = "x86_64")]
    X86_64,
    #[sqlx(rename = "x86_64-mac")]
    X86_64Mac,
}

impl From<Architecture> for ArchitectureRecord {
    fn from(value: Architecture) -> Self {
        match value {
            Architecture::Arm64 => Self::Arm64,
            Architecture::Arm64Mac => Self::Arm64Mac,
            Architecture::I386 => Self::I386,
            Architecture::X86_64 => Self::X86_64,
            Architecture::X86_64Mac => Self::X86_64Mac,
        }
    }
}

impl From<ArchitectureRecord> for Architecture {
    fn from(value: ArchitectureRecord) -> Self {
        match value {
            ArchitectureRecord::Arm64 => Self::Arm64,
            ArchitectureRecord::Arm64Mac => Self::Arm64Mac,
            ArchitectureRecord::I386 => Self::I386,
            ArchitectureRecord::X86_64 => Self::X86_64,
            ArchitectureRecord::X86_64Mac => Self::X86_64Mac,
        }
    }
}

#[derive(sqlx::Type, Clone)]
#[sqlx(type_name = "cpu_configuration")]
pub struct CpuConfigurationRecord {
    pub manufacturer: CpuManufacturerRecord,
    pub architecture: ArchitectureRecord,
    pub millicores: i32,
}

impl From<Cpu> for CpuConfigurationRecord {
    fn from(value: Cpu) -> Self {
        Self {
            manufacturer: value.manufacturer.into(),
            architecture: value.architecture.into(),
            millicores: value.millicores,
        }
    }
}

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(type_name = "gpu_manufacturer", rename_all = "PascalCase")]
pub enum GpuManufacturerRecord {
    Nvidia,
    Amd,
    Habana,
}

impl From<GpuManufacturer> for GpuManufacturerRecord {
    fn from(value: GpuManufacturer) -> Self {
        match value {
            GpuManufacturer::Nvidia => Self::Nvidia,
            GpuManufacturer::Amd => Self::Amd,
            GpuManufacturer::Habana => Self::Habana,
        }
    }
}

impl From<GpuManufacturerRecord> for GpuManufacturer {
    fn from(value: GpuManufacturerRecord) -> Self {
        match value {
            GpuManufacturerRecord::Nvidia => Self::Nvidia,
            GpuManufacturerRecord::Amd => Self::Amd,
            GpuManufacturerRecord::Habana => Self::Habana,
        }
    }
}

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(type_name = "gpu_model")]
pub enum GpuModelRecord {
    #[sqlx(rename = "Radeon Pro V520")]
    RadeonProV520,
    #[sqlx(rename = "Gaudi HL-205")]
    GaudiHL205,
    #[sqlx(rename = "A100")]
    A100,
    #[sqlx(rename = "A10G")]
    A10G,
    #[sqlx(rename = "A30")]
    A30,
    #[sqlx(rename = "B200")]
    B200,
    #[sqlx(rename = "H100")]
    H100,
    #[sqlx(rename = "H200")]
    H200,
    #[sqlx(rename = "L4")]
    L4,
    #[sqlx(rename = "L40")]
    L40,
    #[sqlx(rename = "L40S")]
    L40S,
    #[sqlx(rename = "T4")]
    T4,
    #[sqlx(rename = "T4g")]
    T4g,
    #[sqlx(rename = "V100")]
    V100,
}

impl From<GpuModel> for GpuModelRecord {
    fn from(value: GpuModel) -> Self {
        match value {
            GpuModel::RadeonProV520 => Self::RadeonProV520,
            GpuModel::GaudiHL205 => Self::GaudiHL205,
            GpuModel::A100 => Self::A100,
            GpuModel::A10G => Self::A10G,
            GpuModel::A30 => Self::A30,
            GpuModel::B200 => Self::B200,
            GpuModel::H100 => Self::H100,
            GpuModel::H200 => Self::H200,
            GpuModel::L4 => Self::L4,
            GpuModel::L40 => Self::L40,
            GpuModel::L40S => Self::L40S,
            GpuModel::T4 => Self::T4,
            GpuModel::T4g => Self::T4g,
            GpuModel::V100 => Self::V100,
        }
    }
}

impl From<GpuModelRecord> for GpuModel {
    fn from(value: GpuModelRecord) -> Self {
        match value {
            GpuModelRecord::RadeonProV520 => Self::RadeonProV520,
            GpuModelRecord::GaudiHL205 => Self::GaudiHL205,
            GpuModelRecord::A100 => Self::A100,
            GpuModelRecord::A10G => Self::A10G,
            GpuModelRecord::A30 => Self::A30,
            GpuModelRecord::B200 => Self::B200,
            GpuModelRecord::H100 => Self::H100,
            GpuModelRecord::H200 => Self::H200,
            GpuModelRecord::L4 => Self::L4,
            GpuModelRecord::L40 => Self::L40,
            GpuModelRecord::L40S => Self::L40S,
            GpuModelRecord::T4 => Self::T4,
            GpuModelRecord::T4g => Self::T4g,
            GpuModelRecord::V100 => Self::V100,
        }
    }
}

#[derive(sqlx::Type, Clone)]
#[sqlx(type_name = "gpu_configuration")]
pub struct GpuConfigurationRecord {
    pub manufacturer: GpuManufacturerRecord,
    pub model_name: GpuModelRecord,
    pub memory_mb: i32,
    pub count: i32,
}

impl From<Gpu> for GpuConfigurationRecord {
    fn from(value: Gpu) -> Self {
        Self {
            manufacturer: value.manufacturer.into(),
            model_name: value.model.into(),
            memory_mb: value.memory_mb,
            count: value.count,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct ClusterNodeRecord {
    pub node_id: uuid::Uuid,
    pub cluster_id: uuid::Uuid,
    pub node_status: NodeStatusRecord,
    pub heartbeat_timestamp: DateTime<Utc>,
    pub memory_mb: i32,
    pub cpu: CpuConfigurationRecord,
    pub gpus: Vec<GpuConfigurationRecord>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assigned_job_id: Option<uuid::Uuid>,
    pub reported_job_id: Option<uuid::Uuid>,
}

impl From<ClusterNodeRecord> for ClusterNode {
    fn from(record: ClusterNodeRecord) -> Self {
        Self {
            id: record.node_id.into(),
            cluster_id: record.cluster_id.into(),
            node_status: match record.node_status {
                NodeStatusRecord::Available => NodeStatus::Available,
                NodeStatusRecord::Busy => NodeStatus::Busy,
            },
            heartbeat_timestamp: record.heartbeat_timestamp,
            memory_mb: record.memory_mb,
            cpu: Cpu {
                manufacturer: record.cpu.manufacturer.into(),
                architecture: record.cpu.architecture.into(),
                millicores: record.cpu.millicores,
            },
            gpus: record.gpus.into_iter().map(|gpu| Gpu {
                manufacturer: gpu.manufacturer.into(),
                model: gpu.model_name.into(),
                count: gpu.count,
                memory_mb: gpu.memory_mb,
            }).collect(),
            created_at: record.created_at,
            updated_at: record.updated_at,
            assigned_job_id: record.assigned_job_id.map(Into::into),
            reported_job_id: record.reported_job_id.map(Into::into),
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct ClusterDetailsRecord {
    pub cluster_id: uuid::Uuid,
    pub cluster_name: String,
    pub cluster_description: Option<String>,
    pub total_memory_mb: i64,
    pub used_memory_mb: i64,
    pub total_nodes: i64,
    pub busy_nodes: i64,
    pub total_millicores: i64,
    pub used_millicores: i64,
    pub total_gpus: i64,
    pub used_gpus: i64,
    pub total_running_jobs: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ClusterDetailsRecord> for ClusterDetails {
    fn from(record: ClusterDetailsRecord) -> Self {
        Self {
            id: record.cluster_id.into(),
            name: record.cluster_name,
            description: record.cluster_description,
            created_at: record.created_at,
            updated_at: record.updated_at,
            total_nodes: record.total_nodes,
            busy_nodes: record.busy_nodes,
            memory_info: ClusterMemoryStats {
                total_memory_mb: record.total_memory_mb,
                used_memory_mb: record.used_memory_mb,
            },
            cpu_info: ClusterCpuStats {
                total_millicores: record.total_millicores,
                used_millicores: record.used_millicores,
            },
            gpu_info: ClusterGpuStats {
                total_gpus: record.total_gpus,
                used_gpus: record.used_gpus,
            },
            job_info: ClusterJobStats {
                total_running_jobs: record.total_running_jobs,
            },
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct ApiKeyRecord {
    pub id: uuid::Uuid,
    pub user_id: Option<uuid::Uuid>,
    pub cluster_id: Option<uuid::Uuid>,
    pub prefix: String,
    pub key_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<ApiKeyRecord> for ApiKey {
    fn from(record: ApiKeyRecord) -> Self {
        Self {
            id: record.id.into(),
            user_id: record.user_id.map(|v| v.into()),
            cluster_id: record.cluster_id.map(|v| v.into()),
            prefix: record.prefix,
            key_hash: record.key_hash,
            created_at: record.created_at,
            last_used_at: record.last_used_at,
            expires_at: record.expires_at,
        }
    }
}

//  Training Job Repository Records

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(type_name = "training_job_status", rename_all = "lowercase")]
pub enum TrainingJobStatusRecord {
    Queued,
    Starting,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl From<TrainingJobStatus> for TrainingJobStatusRecord {
    fn from(value: TrainingJobStatus) -> Self {
        match value {
            TrainingJobStatus::Queued => Self::Queued,
            TrainingJobStatus::Starting => Self::Starting,
            TrainingJobStatus::Running => Self::Running,
            TrainingJobStatus::Succeeded => Self::Succeeded,
            TrainingJobStatus::Failed => Self::Failed,
            TrainingJobStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<TrainingJobStatusRecord> for TrainingJobStatus {
    fn from(value: TrainingJobStatusRecord) -> Self {
        match value {
            TrainingJobStatusRecord::Queued => Self::Queued,
            TrainingJobStatusRecord::Starting => Self::Starting,
            TrainingJobStatusRecord::Running => Self::Running,
            TrainingJobStatusRecord::Succeeded => Self::Succeeded,
            TrainingJobStatusRecord::Failed => Self::Failed,
            TrainingJobStatusRecord::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct TrainingJobRecord {
    pub id: Uuid,
    pub name: String,
    pub definition: String,
    pub status: TrainingJobStatusRecord,
    pub node_id: Option<Uuid>,
    pub queue_id: Option<Uuid>,
    pub resource_requirements: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<TrainingJobRecord> for TrainingJob {
    type Error = anyhow::Error;

    fn try_from(value: TrainingJobRecord) -> Result<Self, Self::Error> {
        let resource_requirements = serde_json::from_value(value.resource_requirements)?;
        Ok(Self {
            id: value.id.into(),
            name: value.name,
            definition: value.definition,
            status: value.status.into(),
            node_id: value.node_id.map(|v| v.into()),
            queue_id: value.queue_id.map(Into::into),
            resource_requirements,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}
