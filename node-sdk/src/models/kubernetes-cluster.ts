export type KubernetesCluster = {
  id: string;
  name: string;
  description?: string;
  apiServerUrl: string;
  caFingerprint?: string;
  healthStatus: KubernetesClusterHealthStatus;
  lastHealthCheckAt?: string;
  createdAt: string;
  updatedAt: string;
};

export enum KubernetesClusterHealthStatus {
  Healthy = 'healthy',
  Unreachable = 'unreachable',
}

export type CreateKubernetesClusterInput = {
  name: string;
  description?: string;
  kubeconfig: string;
};

export type UpdateKubernetesClusterInput = {
  clusterId: string;
  name: string;
  description?: string;
  kubeconfig?: string;
};
