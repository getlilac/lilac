use std::sync::Arc;

use super::{
    models::{GetTrainingJobsFilters, TrainingJob, TrainingJobStatus},
    ports::TrainingJobRepository,
};
use crate::{
    domain::{
        cluster::{
            models::NodeId,
            ports::{ClusterRepository, ClusterRepositoryError},
        },
        training_job::{models::JobId, ports::TrainingJobRepositoryError},
    },
    inbound::http::routes::training_jobs::models::CreateTrainingJobRequest,
};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TrainingJobServiceError {
    #[error("training job with {field} {value} already exists")]
    TrainingJobExists { field: String, value: String },
    #[error("training job {0} not found")]
    TrainingJobNotFound(String),
    #[error("invalid training job definition: {0}")]
    InvalidDefinition(#[from] serde_json::Error),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl From<TrainingJobRepositoryError> for TrainingJobServiceError {
    fn from(err: TrainingJobRepositoryError) -> Self {
        match err {
            TrainingJobRepositoryError::Duplicate { field, value } => {
                TrainingJobServiceError::TrainingJobExists { field, value }
            }
            TrainingJobRepositoryError::NotFound(id) => {
                TrainingJobServiceError::TrainingJobNotFound(id)
            }
            TrainingJobRepositoryError::Unknown(err) => TrainingJobServiceError::Unknown(err),
        }
    }
}

impl From<ClusterRepositoryError> for TrainingJobServiceError {
    fn from(err: ClusterRepositoryError) -> Self {
        match err {
            ClusterRepositoryError::Unknown(err) => TrainingJobServiceError::Unknown(err),
            // The other variants don't make sense in this context, so we'll just
            // bubble them up as unknown errors.
            _ => TrainingJobServiceError::Unknown(anyhow::anyhow!(err)),
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TrainingJobService: Send + Sync {
    async fn create(
        &self,
        request: CreateTrainingJobRequest,
    ) -> Result<TrainingJob, TrainingJobServiceError>;
    async fn get_training_jobs(
        &self,
        filters: GetTrainingJobsFilters,
    ) -> Result<Vec<TrainingJob>, TrainingJobServiceError>;
    async fn update_status(
        &self,
        id: &JobId,
        status: TrainingJobStatus,
    ) -> Result<(), TrainingJobServiceError>;
    async fn get_training_job_by_id(
        &self,
        id: &JobId,
    ) -> Result<TrainingJob, TrainingJobServiceError>;
    async fn mark_as_starting(
        &self,
        id: &JobId,
        node_id: &NodeId,
    ) -> Result<(), TrainingJobServiceError>;
    async fn post_logs(&self, id: &JobId, logs: String) -> Result<(), TrainingJobServiceError>;
    async fn cancel(&self, id: &JobId) -> Result<(), TrainingJobServiceError>;
}

pub struct TrainingJobServiceImpl {
    repository: Arc<dyn TrainingJobRepository>,
    cluster_repo: Arc<dyn ClusterRepository>,
}

impl TrainingJobServiceImpl {
    pub fn new(
        repository: Arc<dyn TrainingJobRepository>,
        cluster_repo: Arc<dyn ClusterRepository>,
    ) -> Self {
        Self {
            repository,
            cluster_repo,
        }
    }
}

#[async_trait]
impl TrainingJobService for TrainingJobServiceImpl {
    async fn create(
        &self,
        request: CreateTrainingJobRequest,
    ) -> Result<TrainingJob, TrainingJobServiceError> {
        let job_id = JobId::generate();
        let now = chrono::Utc::now();

        let training_job = TrainingJob {
            id: job_id,
            name: request.name,
            definition: request.definition,
            status: TrainingJobStatus::Queued,
            node_id: None,
            queue_id: Some(request.queue_id),
            resource_requirements: serde_json::from_value(request.resource_requirements)?,
            created_at: now,
            updated_at: now,
        };

        self.repository.create(&training_job).await?;

        Ok(training_job)
    }

    async fn get_training_jobs(
        &self,
        filters: GetTrainingJobsFilters,
    ) -> Result<Vec<TrainingJob>, TrainingJobServiceError> {
        Ok(self.repository.get_training_jobs(filters).await?)
    }

    async fn update_status(
        &self,
        id: &JobId,
        status: TrainingJobStatus,
    ) -> Result<(), TrainingJobServiceError> {
        Ok(self.repository.update_status(id, status).await?)
    }

    async fn get_training_job_by_id(
        &self,
        id: &JobId,
    ) -> Result<TrainingJob, TrainingJobServiceError> {
        Ok(self.repository.get_training_job_by_id(id).await?)
    }

    async fn mark_as_starting(
        &self,
        id: &JobId,
        node_id: &NodeId,
    ) -> Result<(), TrainingJobServiceError> {
        Ok(self.repository.mark_as_starting(id, node_id).await?)
    }

    async fn post_logs(&self, id: &JobId, logs: String) -> Result<(), TrainingJobServiceError> {
        Ok(self.repository.post_logs(id, logs).await?)
    }

    async fn cancel(&self, id: &JobId) -> Result<(), TrainingJobServiceError> {
        let job = self.repository.get_training_job_by_id(id).await?;

        if let Some(node_id) = job.node_id {
            self.cluster_repo
                .clear_assigned_job_id(&node_id)
                .await
                .map_err(|e| TrainingJobServiceError::Unknown(e.into()))?;
        }

        self.repository
            .update_status(id, TrainingJobStatus::Cancelled)
            .await?;

        Ok(())
    }
}
