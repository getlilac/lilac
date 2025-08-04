use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    domain::{
        cluster::{
            errors::ClusterApiKeyRepositoryError,
            models::{
                Cluster, ClusterDetails, ClusterId, ClusterNode, ClusterSummary,
                CreateClusterRequest, NodeId, UpdateNodeStatusRequest,
            },
            ports::{ClusterApiKeyRepository, ClusterRepository, ClusterRepositoryError},
        },
        training_job::models::{JobId, TrainingJob},
        user::models::{ApiKey, ApiKeyId},
    },
    outbound::persistence::postgres::records::{
        ApiKeyRecord, ClusterDetailsRecord, ClusterNodeRecord, ClusterRecord, ClusterSummaryRecord,
        CpuConfigurationRecord, GpuConfigurationRecord, NodeStatusRecord, TrainingJobRecord,
        TrainingJobStatusRecord,
    },
};

#[derive(Clone)]
pub struct PostgresClusterRepository {
    pool: PgPool,
}

impl PostgresClusterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClusterRepository for PostgresClusterRepository {
    async fn create_cluster(
        &self,
        req: &CreateClusterRequest,
    ) -> Result<Cluster, ClusterRepositoryError> {
        let record = sqlx::query_as!(
            ClusterRecord,
            r#"INSERT INTO clusters (cluster_name, cluster_description)
            VALUES ($1, $2)
            RETURNING cluster_id, cluster_name, cluster_description, created_at, updated_at"#,
            req.name,
            req.description,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ClusterRepositoryError::Unknown(e.into()))?;

        Ok(record.into())
    }

    async fn get_cluster_by_id(&self, id: &ClusterId) -> Result<Cluster, ClusterRepositoryError> {
        let record = sqlx::query_as!(
            ClusterRecord,
            r#"
            SELECT c.cluster_id, c.cluster_name, c.cluster_description, c.created_at, c.updated_at
            FROM clusters c
            WHERE c.cluster_id = $1
            "#,
            id.inner(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ClusterRepositoryError::NotFound(id.inner().to_string()),
            _ => ClusterRepositoryError::Unknown(anyhow::anyhow!(e)),
        })?;

        let cluster: Cluster = record.into();

        Ok(cluster)
    }

    async fn get_cluster_details(
        &self,
        id: &ClusterId,
    ) -> Result<ClusterDetails, ClusterRepositoryError> {
        let record = sqlx::query_as!(
            ClusterDetailsRecord,
            r#"SELECT c.cluster_id, c.cluster_name, c.cluster_description, c.created_at, c.updated_at,
                COUNT(DISTINCT n.node_id) AS "total_nodes!: i64",
                COUNT(DISTINCT n.node_id) FILTER (WHERE n.node_status = 'busy') AS "busy_nodes!: i64",
                COUNT(running_jobs.id) AS "total_running_jobs!: i64",
                COALESCE(SUM((n.cpu).millicores), 0) AS "total_millicores!: i64",
                COALESCE(SUM((n.cpu).millicores) FILTER (WHERE n.node_status = 'busy'), 0) AS "used_millicores!: i64",
                COALESCE(SUM(n.memory_mb), 0) AS "total_memory_mb!: i64",
                COALESCE(SUM(n.memory_mb) FILTER (WHERE n.node_status = 'busy'), 0) AS "used_memory_mb!: i64",
                COALESCE(SUM((n.gpu).count), 0) AS "total_gpus!: i64",
                COALESCE(SUM((n.gpu).count) FILTER (WHERE n.node_status = 'busy'), 0) AS "used_gpus!: i64"
            FROM clusters c
            LEFT JOIN cluster_nodes n ON c.cluster_id = n.cluster_id
            LEFT JOIN training_jobs running_jobs ON running_jobs.status = 'running' AND n.node_id = running_jobs.node_id
            WHERE c.cluster_id = $1
            GROUP BY c.cluster_id;
            "#,
            id.inner()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ClusterRepositoryError::NotFound(id.to_string()),
            _ => ClusterRepositoryError::Unknown(anyhow::anyhow!(e)),
        })?;

        Ok(record.into())
    }

    async fn list_clusters(&self) -> Result<Vec<ClusterSummary>, ClusterRepositoryError> {
        let records = sqlx::query_as!(
            ClusterSummaryRecord,
            r#"
            SELECT c.cluster_id, c.cluster_name, c.cluster_description, c.created_at, c.updated_at,
                COUNT(DISTINCT n.node_id) AS "total_nodes!: i64",
                COUNT(DISTINCT n.node_id) FILTER (WHERE n.node_status = 'busy') AS "busy_nodes!: i64",
                COUNT(running_jobs.id) AS "total_running_jobs!: i64"
            FROM clusters c
            LEFT JOIN cluster_nodes n ON c.cluster_id = n.cluster_id
            LEFT JOIN training_jobs running_jobs ON running_jobs.status = 'running' AND n.node_id = running_jobs.node_id
            GROUP BY c.cluster_id;
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e: sqlx::Error| ClusterRepositoryError::Unknown(anyhow::anyhow!(e)))?;

        let clusters: Vec<ClusterSummary> = records.into_iter().map(|r| r.into()).collect();
        Ok(clusters)
    }

    async fn list_cluster_jobs(
        &self,
        cluster_id: &ClusterId,
    ) -> Result<Vec<TrainingJob>, ClusterRepositoryError> {
        let records = sqlx::query_as!(
            TrainingJobRecord,
            r#"
            SELECT id, name, definition, status AS "status: TrainingJobStatusRecord", node_id, queue_id, resource_requirements, created_at, updated_at
            FROM training_jobs
            WHERE node_id = ANY(SELECT node_id FROM cluster_nodes WHERE cluster_id = $1)
            "#,
            cluster_id.inner(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e: sqlx::Error| ClusterRepositoryError::Unknown(anyhow::anyhow!(e)))?;
        Ok(records
            .into_iter()
            .map(|v| v.try_into())
            .collect::<Result<Vec<_>, anyhow::Error>>()?)
    }

    async fn delete_cluster(&self, id: &ClusterId) -> Result<(), ClusterRepositoryError> {
        sqlx::query!("DELETE FROM clusters WHERE cluster_id = $1", id.inner())
            .execute(&self.pool)
            .await
            .map_err(|e: sqlx::Error| ClusterRepositoryError::Unknown(anyhow::anyhow!(e)))?;
        Ok(())
    }

    async fn list_all_nodes(&self) -> Result<Vec<ClusterNode>, ClusterRepositoryError> {
        let records = sqlx::query_as!(
            ClusterNodeRecord,
            r#"
            SELECT node_id, cluster_id, node_status as "node_status: NodeStatusRecord", heartbeat_timestamp, memory_mb, cpu as "cpu: CpuConfigurationRecord", gpu as "gpu: GpuConfigurationRecord", created_at, updated_at, assigned_job_id, reported_job_id
            FROM cluster_nodes
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e: sqlx::Error| ClusterRepositoryError::Unknown(anyhow::anyhow!(e)))?;

        let nodes: Vec<ClusterNode> = records.into_iter().map(|r| r.into()).collect();
        Ok(nodes)
    }

    async fn list_cluster_nodes(
        &self,
        id: &ClusterId,
    ) -> Result<Vec<ClusterNode>, ClusterRepositoryError> {
        let records = sqlx::query_as!(
            ClusterNodeRecord,
            r#"
            SELECT node_id, cluster_id, node_status as "node_status: NodeStatusRecord", heartbeat_timestamp, memory_mb, cpu as "cpu: CpuConfigurationRecord", gpu as "gpu: GpuConfigurationRecord", created_at, updated_at, assigned_job_id, reported_job_id
            FROM cluster_nodes
            WHERE cluster_id = $1
            "#,
            id.inner(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e: sqlx::Error| ClusterRepositoryError::Unknown(anyhow::anyhow!(e)))?;

        let clusters: Vec<ClusterNode> = records.into_iter().map(|r| r.into()).collect();
        Ok(clusters)
    }
    async fn get_cluster_node_by_id(
        &self,
        id: &NodeId,
    ) -> Result<ClusterNode, ClusterRepositoryError> {
        let record = sqlx::query_as!(
            ClusterNodeRecord,
            r#"
            SELECT node_id, cluster_id, node_status as "node_status: NodeStatusRecord", heartbeat_timestamp, memory_mb, cpu as "cpu: CpuConfigurationRecord", gpu as "gpu: GpuConfigurationRecord", created_at, updated_at, assigned_job_id, reported_job_id
            FROM cluster_nodes
            WHERE node_id = $1
            "#,
            id.inner(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e: sqlx::Error| ClusterRepositoryError::Unknown(anyhow::anyhow!(e)))?;
        Ok(record.into())
    }
    async fn update_cluster_node_status(
        &self,
        req: &UpdateNodeStatusRequest,
    ) -> Result<ClusterNode, ClusterRepositoryError> {
        let record = sqlx::query_as!(
            ClusterNodeRecord,
            r#"
            INSERT INTO cluster_nodes (node_id, cluster_id, node_status, heartbeat_timestamp, memory_mb, cpu, gpu, reported_job_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (node_id) DO UPDATE SET
                    node_status = EXCLUDED.node_status,
                    heartbeat_timestamp = EXCLUDED.heartbeat_timestamp,
                    reported_job_id = EXCLUDED.reported_job_id,
                    updated_at = NOW()
                RETURNING node_id, cluster_id, node_status as "node_status: NodeStatusRecord", heartbeat_timestamp, memory_mb, cpu as "cpu: CpuConfigurationRecord", gpu as "gpu: GpuConfigurationRecord", created_at, updated_at, assigned_job_id, reported_job_id;
            "#,
            req.node_id.inner(),
            req.cluster_id.inner(),
            match req.job_info {
                Some(_) => NodeStatusRecord::Busy,
                None => NodeStatusRecord::Available,
            } as _,
            req.heartbeat_timestamp,
            req.memory_info,
            CpuConfigurationRecord::from(req.cpu_info.clone()) as _,
            req.gpu_info
                .clone()
                .map(GpuConfigurationRecord::from) as _,
            req.job_info
                .as_ref()
                .map(|info| info.current_job_id)
                .map(|id| id.into_inner()),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e: sqlx::Error| ClusterRepositoryError::Unknown(anyhow::anyhow!(e)))?;
        Ok(record.into())
    }
    async fn delete_cluster_node(&self, node_id: &NodeId) -> Result<(), ClusterRepositoryError> {
        sqlx::query!(
            "DELETE FROM cluster_nodes WHERE node_id = $1",
            node_id.inner()
        )
        .execute(&self.pool)
        .await
        .map_err(|e: sqlx::Error| ClusterRepositoryError::Unknown(anyhow::anyhow!(e)))?;
        Ok(())
    }

    async fn clear_assigned_job_id(&self, node_id: &NodeId) -> Result<(), ClusterRepositoryError> {
        sqlx::query!(
            "UPDATE cluster_nodes SET assigned_job_id = NULL WHERE node_id = $1",
            node_id.inner()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ClusterRepositoryError::Unknown(anyhow::anyhow!(e)))?;

        Ok(())
    }

    async fn assign_job_to_node(
        &self,
        node_id: &NodeId,
        job_id: &JobId,
    ) -> Result<(), ClusterRepositoryError> {
        let result = sqlx::query!(
            "UPDATE cluster_nodes SET assigned_job_id = $1 WHERE node_id = $2",
            job_id.inner(),
            node_id.inner()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ClusterRepositoryError::Unknown(anyhow::anyhow!(e)))?;

        if result.rows_affected() == 0 {
            return Err(ClusterRepositoryError::NotFound(
                node_id.inner().to_string(),
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl ClusterApiKeyRepository for PostgresClusterRepository {
    async fn create_api_key(&self, key: &ApiKey) -> Result<(), ClusterApiKeyRepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO api_keys (id, user_id, cluster_id, prefix, key_hash, created_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            key.id.inner(),
            key.user_id.map(|v| v.into_inner()),
            key.cluster_id.map(|v| v.into_inner()),
            key.prefix,
            key.key_hash,
            key.created_at,
            key.expires_at,
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|err| ClusterApiKeyRepositoryError::Unknown(anyhow::anyhow!(err)))
    }

    async fn find_cluster_by_api_key_hash(
        &self,
        key_hash: &str,
    ) -> Result<Cluster, ClusterApiKeyRepositoryError> {
        let record = sqlx::query_as!(
            ClusterRecord,
            r#"
            SELECT c.cluster_id, c.cluster_name, c.cluster_description, c.created_at, c.updated_at
            FROM clusters c
            JOIN api_keys ak ON c.cluster_id = ak.cluster_id
            WHERE ak.key_hash = $1 AND (ak.expires_at IS NULL OR ak.expires_at > now())
            "#,
            key_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => ClusterApiKeyRepositoryError::NotFound,
            _ => ClusterApiKeyRepositoryError::Unknown(anyhow::anyhow!(err)),
        })?;

        Ok(record.into())
    }

    async fn get_api_key(
        &self,
        cluster_id: &ClusterId,
        key_id: &ApiKeyId,
    ) -> Result<ApiKey, ClusterApiKeyRepositoryError> {
        let record = sqlx::query_as!(
            ApiKeyRecord,
            r#"
            SELECT id, user_id, cluster_id, prefix, key_hash, created_at, last_used_at, expires_at
            FROM api_keys
            WHERE cluster_id = $1 AND id = $2
            "#,
            cluster_id.inner(),
            key_id.inner(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| ClusterApiKeyRepositoryError::Unknown(anyhow::anyhow!(err)))?;

        Ok(record.into())
    }

    async fn list_api_keys_for_cluster(
        &self,
        cluster_id: &ClusterId,
    ) -> Result<Vec<ApiKey>, ClusterApiKeyRepositoryError> {
        let records = sqlx::query_as!(
            ApiKeyRecord,
            r#"
            SELECT id, user_id, cluster_id, prefix, key_hash, created_at, last_used_at, expires_at
            FROM api_keys
            WHERE cluster_id = $1
            "#,
            cluster_id.inner()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| ClusterApiKeyRepositoryError::Unknown(anyhow::anyhow!(err)))?;

        Ok(records.into_iter().map(ApiKey::from).collect())
    }

    async fn delete_api_key(
        &self,
        cluster_id: &ClusterId,
        key_id: &ApiKeyId,
    ) -> Result<(), ClusterApiKeyRepositoryError> {
        let result = sqlx::query!(
            "DELETE FROM api_keys WHERE cluster_id = $1 AND id = $2",
            cluster_id.inner(),
            key_id.inner()
        )
        .execute(&self.pool)
        .await
        .map_err(|err| ClusterApiKeyRepositoryError::Unknown(anyhow::anyhow!(err)))?;

        if result.rows_affected() == 0 {
            return Err(ClusterApiKeyRepositoryError::NotFound);
        }

        Ok(())
    }
}
