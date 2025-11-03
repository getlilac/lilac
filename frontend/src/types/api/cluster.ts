import { ResourceRequirements } from './job';

export interface Cluster {
  clusterId: string;
  clusterName: string;
  clusterDescription?: string;
}

export interface ClusterSummary {
  clusterId: string;
  clusterName: string;
  clusterDescription?: string;
  totalNodes: number;
  busyNodes: number;
  totalRunningJobs: number;
}

export interface ClusterInfo {
  clusterId: string;
  clusterName: string;
  clusterDescription?: string;
  totalNodes: number;
  busyNodes: number;
  memoryInfo: {
    totalMemoryMb: number;
    usedMemoryMb: number;
  };
  cpuInfo: {
    totalMillicores: number;
    usedMillicores: number;
  };
  gpuInfo: {
    totalGpus: number;
    usedGpus: number;
  };
  jobInfo: {
    totalRunningJobs: number;
  };
  createdAt: string;
  updatedAt: string;
}

export interface ClusterJob {
  jobId: string;
  jobName: string;
  jobStatus: string;
  nodeId?: string;
  queueId: string;
  resourceRequirements: ResourceRequirements;
  createdAt: string;
  updatedAt: string;
}

export interface ClusterApiKey {
  id: string;
  clusterId: string;
  prefix: string;
  createdAt: string;
  lastUsedAt?: string;
  expiresAt?: string;
}

export interface Gpu {
  manufacturer: string;
  model: string;
  count: number;
  memoryMb: number;
}

export interface ClusterNode {
  id: string;
  clusterId: string;
  nodeStatus: 'busy' | 'available';
  lastHeartbeat: string;
  memoryMb: number;
  cpu: {
    manufacturer: string;
    architecture: string;
    millicores: number;
  };
  gpus: Gpu[];
}
