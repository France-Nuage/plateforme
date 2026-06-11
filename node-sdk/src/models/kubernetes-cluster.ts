export type KubernetesCluster = {
  id: string;
  name: string;
  description?: string;
  apiServerUrl: string;
  caFingerprint?: string;
  kubernetesVersion?: string;
  platform?: string;
  healthStatus: KubernetesClusterHealthStatus;
  lastHealthCheckAt?: string;
  createdAt: string;
  updatedAt: string;
  /** Labels attached to the cluster, ordered by key then value. */
  labels: KubernetesLabel[];
};

/**
 * A cluster label: a reusable key/value pair (e.g. availability=ft) attached
 * to clusters and matched against managed-service deploy targets. Labels with
 * `system = true` are owned by the control plane and read-only via the API.
 */
export type KubernetesLabel = {
  id: string;
  key: string;
  value: string;
  system: boolean;
  createdAt: string;
  updatedAt: string;
};

/**
 * Minimal reference to a managed service whose deploy_target requires a
 * label. Surfaced to the operator before a label is detached or deleted.
 */
export type ManagedServiceRef = {
  id: string;
  slug: string;
  name: string;
};

export type CreateKubernetesLabelInput = {
  /** Max 49 chars, charset [a-zA-Z0-9-], case-insensitive. */
  key: string;
  /** Max 49 chars, charset [a-zA-Z0-9-], case-insensitive. */
  value: string;
};

export enum KubernetesClusterHealthStatus {
  Healthy = 'healthy',
  Unreachable = 'unreachable',
}

export type CreateKubernetesClusterInput = {
  name: string;
  description?: string;
  kubeconfig: string;
  /**
   * IDs of existing user-managed labels to attach at creation. An unknown or
   * system label rejects the whole request before any cluster is created.
   */
  labelIds?: string[];
};

export type UpdateKubernetesClusterInput = {
  clusterId: string;
  name: string;
  description?: string;
  kubeconfig?: string;
};
