use crate::{
    domain::agent::{
        models::{HeartbeatRequest, JobInfo, JobStatus, Gpu, GpuManufacturer, GpuModel},
        ports::{ControlPlaneApi, JobExecutor, SystemMonitor},
    },
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::{
    sync::Notify,
    task::JoinHandle,
    time::{self, MissedTickBehavior},
};
use uuid::Uuid;

pub struct Daemon<C, S, J>
where
    C: ControlPlaneApi + Clone + 'static,
    S: SystemMonitor,
    J: JobExecutor + Clone + 'static,
{
    control_plane: Arc<C>,
    system_monitor: Arc<S>,
    job_executor: Arc<J>,
    heartbeat_interval: Duration,
    current_job: Arc<Mutex<Option<JobInfo>>>,
    job_handle: Arc<Mutex<Option<(Uuid, JoinHandle<()>)>>>,
    node_id: Uuid,
    heartbeat_now: Arc<Notify>,
}

impl<C, S, J> Daemon<C, S, J>
where
    C: ControlPlaneApi + Clone + 'static,
    S: SystemMonitor,
    J: JobExecutor + Clone + 'static,
{
    pub fn new(control_plane: C, system_monitor: S, job_executor: J, node_id: Uuid) -> Self {
        Self {
            control_plane: Arc::new(control_plane),
            system_monitor: Arc::new(system_monitor),
            job_executor: Arc::new(job_executor),
            heartbeat_interval: Duration::from_secs(30),
            current_job: Arc::new(Mutex::new(None)),
            job_handle: Arc::new(Mutex::new(None)),
            node_id,
            heartbeat_now: Arc::new(Notify::new()),
        }
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        println!("[DAEMON] Starting Lilac agent daemon...");

        let resources = self
            .system_monitor
            .get_node_resources()
            .await
            .map_err(|e| anyhow::Error::new(e).context("Failed to get node resources"))?;
        println!("[DAEMON] Discovered resources: {:?}", resources);

        let mut interval = time::interval(self.heartbeat_interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    println!("[DAEMON] Sending scheduled heartbeat...");
                },
                _ = self.heartbeat_now.notified() => {
                    println!("[DAEMON] Job status changed, sending immediate heartbeat...");
                }
            }

            let current_job_info = self.current_job.lock().unwrap().clone();
            println!("[DAEMON] GPU resources: {:?}", resources.gpus);

            // Aggregate GPUs by (manufacturer, model, memory_mb)
            let mut gpu_counts: HashMap<(GpuManufacturer, GpuModel, i32), i32> = HashMap::new();
            for g in &resources.gpus {
                let key = (g.manufacturer.clone(), g.model.clone(), g.memory_mb);
                *gpu_counts.entry(key).or_insert(0) += g.count.max(1);
            }

            let gpu_info: Vec<Gpu> = gpu_counts
                .into_iter()
                .map(|((manufacturer, model, memory_mb), count)| Gpu {
                    manufacturer,
                    model,
                    count,
                    memory_mb,
                })
                .collect();

            println!("[DAEMON] GPU info: {:?}", gpu_info);

            let request = HeartbeatRequest {
                memory_info: resources.memory_mb,
                cpu_info: resources.cpu.clone(),
                gpu_info,
                job_info: current_job_info,
            };

            let response = self
                .control_plane
                .send_heartbeat(self.node_id, request)
                .await;

            match response {
                Ok(response) => {
                    let mut current_job_guard = self.current_job.lock().unwrap();
                    let assigned_job_id = response.assigned_job.as_ref().map(|j| j.id);
                    let current_job_id = current_job_guard.as_ref().map(|j| j.current_job_id);

                    if current_job_id != assigned_job_id {
                        println!(
                            "[DAEMON] Reconciliation needed. Current: {:?}, Assigned: {:?}",
                            current_job_id, assigned_job_id
                        );

                        if let Some((job_id, handle)) = self.job_handle.lock().unwrap().take() {
                            println!("[DAEMON] Aborting previous job {}.", job_id);
                            handle.abort();
                            let job_executor = self.job_executor.clone();
                            tokio::spawn(async move {
                                if let Err(e) = job_executor.stop_job(&job_id.to_string()).await {
                                    eprintln!(
                                        "[DAEMON] Error stopping job container for job {}: {}",
                                        job_id, e
                                    );
                                }
                            });
                        }

                        if let Some(assigned_job) = response.assigned_job {
                            let job_id = assigned_job.id;
                            println!("[DAEMON] Starting new job with ID: {}", job_id);
                            let new_job_info = JobInfo {
                                current_job_id: job_id,
                                status: JobStatus::Acknowledged,
                            };
                            *current_job_guard = Some(new_job_info);

                            let executor = self.job_executor.clone();
                            let current_job_clone = self.current_job.clone();
                            let job_handle_clone = self.job_handle.clone();
                            let resources_clone = resources.clone();
                            let heartbeat_now_clone = self.heartbeat_now.clone();

                            let handle = tokio::spawn(async move {
                                if let Some(job_info) = &mut *current_job_clone.lock().unwrap() {
                                    job_info.status = JobStatus::Starting;
                                }
                                heartbeat_now_clone.notify_one();

                                if let Some(job_info) = &mut *current_job_clone.lock().unwrap() {
                                    job_info.status = JobStatus::Running;
                                }

                                let final_status =
                                    match executor.run_job(assigned_job, &resources_clone).await {
                                        Ok(0) => {
                                            println!("[JOB {}] Execution finished successfully.", job_id);
                                            JobStatus::Succeeded
                                        }
                                        Ok(exit_code) => {
                                            eprintln!("[JOB {}] Execution finished with a non-zero exit code: {}", job_id, exit_code);
                                            JobStatus::Failed
                                        }
                                        Err(e) => {
                                            eprintln!("[JOB {}] Execution failed: {}", job_id, e);
                                            JobStatus::Failed
                                        }
                                    };

                                if let Some(job_info) = &mut *current_job_clone.lock().unwrap() {
                                    job_info.status = final_status;
                                }
                                heartbeat_now_clone.notify_one();
                            });
                            *job_handle_clone.lock().unwrap() = Some((job_id, handle));
                        } else {
                            *current_job_guard = None;
                            println!("[DAEMON] Node is now available.");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[DAEMON] Error sending heartbeat: {}. Will retry.", e);
                }
            }
        }
    }
}