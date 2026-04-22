use std::sync::Arc;

use tracing::{error, info};

use crate::{
    domain::{
        cluster::ports::ClusterRepository, queue::ports::QueueRepository,
        training_job::ports::TrainingJobRepository,
    },
    outbound::scheduler::agent_adapter::{AgentSchedulerAdapter, AgentSchedulerError},
};
use chrono::Utc;
use thiserror::Error;

use crate::domain::{
    cluster::ports::ClusterRepositoryError, queue::ports::QueueRepositoryError,
    training_job::ports::TrainingJobRepositoryError,
};

#[derive(Debug, Error)]
pub enum SchedulerServiceError {
    #[error(transparent)]
    Job(#[from] TrainingJobRepositoryError),
    #[error(transparent)]
    Queue(#[from] QueueRepositoryError),
    #[error(transparent)]
    Cluster(#[from] ClusterRepositoryError),
    #[error(transparent)]
    Agent(#[from] AgentSchedulerError),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

pub struct SchedulerService {
    job_repo: Arc<dyn TrainingJobRepository>,
    queue_repo: Arc<dyn QueueRepository>,
    cluster_repo: Arc<dyn ClusterRepository>,
    agent_adapter: Arc<AgentSchedulerAdapter>,
}

impl SchedulerService {
    pub fn new(
        job_repo: Arc<dyn TrainingJobRepository>,
        queue_repo: Arc<dyn QueueRepository>,
        cluster_repo: Arc<dyn ClusterRepository>,
        agent_adapter: Arc<AgentSchedulerAdapter>,
    ) -> Self {
        Self {
            job_repo,
            queue_repo,
            cluster_repo,
            agent_adapter,
        }
    }

    async fn cleanup_dead_nodes(&self) -> Result<(), SchedulerServiceError> {
        info!("Running dead node cleanup...");
        let nodes = self.cluster_repo.list_all_nodes().await?;
        for node in nodes {
            let since_heartbeat = Utc::now() - node.heartbeat_timestamp;
            if since_heartbeat > chrono::Duration::seconds(90) {
                info!("Found dead node {}. Cleaning up.", node.id);

                if let Some(job_id) = node.assigned_job_id {
                    info!(
                        "Re-queueing assigned job {} from dead node {}",
                        job_id, node.id
                    );
                    self.job_repo.reset_job_status(&job_id).await?;
                }

                if let Some(job_id) = node.reported_job_id {
                    info!(
                        "Re-queueing reported job {} from dead node {}",
                        job_id, node.id
                    );
                    self.job_repo.reset_job_status(&job_id).await?;
                }

                self.cluster_repo.delete_cluster_node(&node.id).await?;
            }
        }
        Ok(())
    }

    async fn cleanup_stale_starting_jobs(&self) -> Result<(), SchedulerServiceError> {
        info!("Running stale job cleanup...");
        let jobs = self
            .job_repo
            .get_jobs_by_status(super::super::training_job::models::TrainingJobStatus::Starting)
            .await?;

        for job in jobs {
            let mut requeue = false;
            let mut cancel = false;

            if let Some(node_id) = job.node_id {
                if self
                    .cluster_repo
                    .get_cluster_node_by_id(&node_id)
                    .await
                    .is_err()
                {
                    info!(
                        "Found stale job {} assigned to non-existent node {}. Re-queueing.",
                        job.id, node_id
                    );
                    requeue = true;
                }
            }

            if let Some(queue_id) = &job.queue_id {
                if self.queue_repo.get_queue_by_id(queue_id).await.is_err() {
                    info!(
                        "Found stale job {} assigned to non-existent queue {}. Cancelling.",
                        job.id, queue_id
                    );
                    cancel = true;
                }
            }

            if cancel {
                self.job_repo
                    .update_status(
                        &job.id,
                        super::super::training_job::models::TrainingJobStatus::Cancelled,
                    )
                    .await?;
            } else if requeue {
                self.job_repo.reset_job_status(&job.id).await?;
            }
        }
        Ok(())
    }

    async fn cleanup_preempted_jobs(&self) -> Result<(), SchedulerServiceError> {
        info!("Running preempted job cleanup...");
        let nodes = self.cluster_repo.list_all_nodes().await?;
        for node in nodes {
            if node.assigned_job_id.is_none() {
                if let Some(reported_job_id) = node.reported_job_id {
                    let job = self
                        .job_repo
                        .get_training_job_by_id(&reported_job_id)
                        .await?;
                    if !matches!(
                        job.status,
                        super::super::training_job::models::TrainingJobStatus::Succeeded
                            | super::super::training_job::models::TrainingJobStatus::Failed
                            | super::super::training_job::models::TrainingJobStatus::Cancelled
                    ) {
                        info!(
                            "Found preempted job {} on node {}. Re-queueing.",
                            job.id, node.id
                        );
                        self.job_repo.reset_job_status(&job.id).await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn cleanup_orphaned_queued_jobs(&self) -> Result<(), SchedulerServiceError> {
        info!("Running orphaned queued job cleanup...");
        let jobs = self
            .job_repo
            .get_jobs_by_status(super::super::training_job::models::TrainingJobStatus::Queued)
            .await?;
        for job in jobs {
            if job.queue_id.is_none() {
                info!("Found orphaned queued job {}. Cancelling.", job.id);
                self.job_repo
                    .update_status(
                        &job.id,
                        super::super::training_job::models::TrainingJobStatus::Cancelled,
                    )
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn run_cycle(&self) -> Result<(), SchedulerServiceError> {
        info!("Starting scheduler cycle");

        if let Err(e) = self.cleanup_dead_nodes().await {
            error!("Error during dead node cleanup: {}", e);
        }
        if let Err(e) = self.cleanup_stale_starting_jobs().await {
            error!("Error during stale starting job cleanup: {}", e);
        }
        if let Err(e) = self.cleanup_preempted_jobs().await {
            error!("Error during preempted job cleanup: {}", e);
        }
        if let Err(e) = self.cleanup_orphaned_queued_jobs().await {
            error!("Error during orphaned queued job cleanup: {}", e);
        }

        let queues = self.queue_repo.get_all_queues_sorted().await?;

        info!("Processing {} queues", queues.len());

        for queue in queues {
            let queued_jobs = self.job_repo.get_queued_jobs_for_queue(&queue.id).await?;

            if queued_jobs.is_empty() {
                continue;
            }

            info!(
                "Found {} queued jobs in queue '{}'",
                queued_jobs.len(),
                queue.name
            );

            for job in queued_jobs {
                info!("Processing job {}", job.id);
                let mut scheduled = false;

                for cluster_id in &queue.cluster_targets {
                    match self
                        .agent_adapter
                        .find_and_allocate_job(&job.id, cluster_id, &job.resource_requirements)
                        .await
                    {
                        Ok(Some(node_id)) => {
                            info!("Successfully allocated job {} to node {}", job.id, node_id);
                            self.job_repo.mark_as_starting(&job.id, &node_id).await?;
                            scheduled = true;
                            break; // Break from cluster loop, move to next job
                        }
                        Ok(None) => {
                            // This is the expected case when no node is found, just info log.
                            info!(
                                "No suitable node found for job {} on cluster {}",
                                job.id, cluster_id
                            );
                        }
                        Err(e) => {
                            // This is an unexpected error during the node search.
                            error!(
                                "Error finding suitable node for job {} on cluster {}: {}",
                                job.id, cluster_id, e
                            );
                        }
                    }
                }

                if !scheduled {
                    info!(
                        "Could not schedule job {} on any cluster in queue '{}'",
                        job.id, queue.name
                    );
                }
            }
        }
        info!("Scheduler cycle finished");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        cluster::{
            models::{ClusterId, ClusterNode},
            ports::MockClusterRepository,
        },
        queue::{models::Queue, ports::MockQueueRepository},
        training_job::{
            models::{TrainingJob, TrainingJobStatus},
            ports::MockTrainingJobRepository,
        },
    };
    use mockall::predicate::{eq};

    fn build_service(
        job_repo: Arc<MockTrainingJobRepository>,
        queue_repo: Arc<MockQueueRepository>,
        cluster_repo: Arc<MockClusterRepository>,
    ) -> SchedulerService {
        let agent_adapter = Arc::new(AgentSchedulerAdapter::new(cluster_repo.clone()));
        SchedulerService::new(job_repo, queue_repo, cluster_repo, agent_adapter)
    }

    #[tokio::test]
    async fn cleanup_dead_nodes_requeues_jobs_and_deletes_stale_node() {

        let mut stale_node = ClusterNode::new_mock();
        stale_node.heartbeat_timestamp = Utc::now() - chrono::Duration::seconds(91);
        stale_node.assigned_job_id = Some(crate::domain::training_job::models::JobId::generate());
        stale_node.reported_job_id = Some(crate::domain::training_job::models::JobId::generate());

        let assigned_job_id = stale_node.assigned_job_id.clone().unwrap();
        let reported_job_id = stale_node.reported_job_id.clone().unwrap();
        let stale_node_id = stale_node.id.clone();

        let mut job_repo = MockTrainingJobRepository::new();
        job_repo
            .expect_reset_job_status()
            .with(eq(assigned_job_id.clone()))
            .times(1)
            .returning(|_| Ok(()));
        job_repo
            .expect_reset_job_status()
            .with(eq(reported_job_id.clone()))
            .times(1)
            .returning(|_| Ok(()));

        let queue_repo = MockQueueRepository::new();

        let mut cluster_repo = MockClusterRepository::new();
        cluster_repo
            .expect_list_all_nodes()
            .times(1)
            .returning(move || Ok(vec![stale_node.clone()]));
        cluster_repo
            .expect_delete_cluster_node()
            .with(eq(stale_node_id.clone()))
            .times(1)
            .returning(|_| Ok(()));

        let service = build_service(
            Arc::new(job_repo),
            Arc::new(queue_repo),
            Arc::new(cluster_repo),
        );

        service.cleanup_dead_nodes().await.unwrap();
    }

    #[tokio::test]
    async fn cleanup_stale_starting_jobs_cancels_when_queue_is_missing() {
        let mut job = TrainingJob::new_mock();
        job.status = TrainingJobStatus::Starting;
        job.node_id = Some(crate::domain::cluster::models::NodeId::generate());
        job.queue_id = Some(crate::domain::queue::models::QueueId::generate());

        let job_id = job.id.clone();
        let node_id = job.node_id.clone().unwrap();
        let queue_id = job.queue_id.clone().unwrap();

        let mut job_repo = MockTrainingJobRepository::new();
        job_repo
            .expect_get_jobs_by_status()
            .with(eq(TrainingJobStatus::Starting))
            .times(1)
            .returning(move |_| Ok(vec![job.clone()]));
        job_repo
            .expect_update_status()
            .with(eq(job_id.clone()), eq(TrainingJobStatus::Cancelled))
            .times(1)
            .returning(|_, _| Ok(()));
        job_repo.expect_reset_job_status().times(0);

        let mut queue_repo = MockQueueRepository::new();
        queue_repo
            .expect_get_queue_by_id()
            .with(eq(queue_id.clone()))
            .times(1)
            .returning(|_| Err(QueueRepositoryError::NotFound("missing-queue".to_string())));

        let mut cluster_repo = MockClusterRepository::new();
        cluster_repo
            .expect_get_cluster_node_by_id()
            .with(eq(node_id.clone()))
            .times(1)
            .returning(|_| Ok(ClusterNode::new_mock()));

        let service = build_service(
            Arc::new(job_repo),
            Arc::new(queue_repo),
            Arc::new(cluster_repo),
        );

        service.cleanup_stale_starting_jobs().await.unwrap();
    }

    #[tokio::test]
    async fn cleanup_preempted_jobs_requeues_non_terminal_reported_job() {
        let job = TrainingJob {
            status: TrainingJobStatus::Running,
            ..TrainingJob::new_mock()
        };
        let job_id = job.id.clone();

        let mut node = ClusterNode::new_mock();
        node.assigned_job_id = None;
        node.reported_job_id = Some(job_id.clone());

        let mut job_repo = MockTrainingJobRepository::new();
        job_repo
            .expect_get_training_job_by_id()
            .with(eq(job_id.clone()))
            .times(1)
            .returning(move |_| Ok(job.clone()));
        job_repo
            .expect_reset_job_status()
            .with(eq(job_id.clone()))
            .times(1)
            .returning(|_| Ok(()));

        let queue_repo = MockQueueRepository::new();

        let mut cluster_repo = MockClusterRepository::new();
        cluster_repo
            .expect_list_all_nodes()
            .times(1)
            .returning(move || Ok(vec![node.clone()]));

        let service = build_service(
            Arc::new(job_repo),
            Arc::new(queue_repo),
            Arc::new(cluster_repo),
        );

        service.cleanup_preempted_jobs().await.unwrap();
    }

    #[tokio::test]
    async fn run_cycle_marks_job_starting_when_agent_finds_capacity() {
        let cluster_id = ClusterId::generate();

        let mut queue = Queue::new_mock();
        queue.cluster_targets = vec![cluster_id.clone()];
        let queue_id = queue.id.clone();

        let mut job = TrainingJob::new_mock();
        job.queue_id = Some(queue_id.clone());
        let job_id = job.id.clone();
        let resource_requirements = job.resource_requirements.clone();

        let mut schedulable_node = ClusterNode::new_mock();
        schedulable_node.cluster_id = cluster_id.clone();
        schedulable_node.cpu.millicores = resource_requirements.cpu_millicores;
        schedulable_node.memory_mb = resource_requirements.memory_mb;
        let node_id = schedulable_node.id.clone();

        let mut job_repo = MockTrainingJobRepository::new();
        job_repo
            .expect_get_jobs_by_status()
            .with(eq(TrainingJobStatus::Starting))
            .times(1)
            .returning(|_| Ok(vec![]));
        job_repo
            .expect_get_jobs_by_status()
            .with(eq(TrainingJobStatus::Queued))
            .times(1)
            .returning(|_| Ok(vec![]));
        job_repo
            .expect_get_queued_jobs_for_queue()
            .with(eq(queue_id.clone()))
            .times(1)
            .returning(move |_| Ok(vec![job.clone()]));
        job_repo
            .expect_mark_as_starting()
            .with(eq(job_id.clone()), eq(node_id.clone()))
            .times(1)
            .returning(|_, _| Ok(()));

        let mut queue_repo = MockQueueRepository::new();
        queue_repo
            .expect_get_all_queues_sorted()
            .times(1)
            .returning(move || Ok(vec![queue.clone()]));

        let mut cluster_repo = MockClusterRepository::new();
        cluster_repo
            .expect_list_all_nodes()
            .times(2)
            .returning(|| Ok(vec![]));
        cluster_repo
            .expect_list_cluster_nodes()
            .with(eq(cluster_id.clone()))
            .times(1)
            .returning(move |_| Ok(vec![schedulable_node.clone()]));
        cluster_repo
            .expect_assign_job_to_node()
            .with(eq(node_id.clone()), eq(job_id.clone()))
            .times(1)
            .returning(|_, _| Ok(()));

        let service = build_service(
            Arc::new(job_repo),
            Arc::new(queue_repo),
            Arc::new(cluster_repo),
        );

        service.run_cycle().await.unwrap();
    }
}
