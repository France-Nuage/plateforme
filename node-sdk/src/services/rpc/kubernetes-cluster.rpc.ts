import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { KubernetesClusterProto } from '../../generated/rpc/kubernetes';
import { KubernetesClustersClient } from '../../generated/rpc/kubernetes.client';
import {
  CreateKubernetesClusterInput,
  KubernetesCluster,
  KubernetesClusterHealthStatus,
  UpdateKubernetesClusterInput,
} from '../../models';
import { KubernetesClusterService } from '../api';

export class KubernetesClusterRpcService implements KubernetesClusterService {
  private client: KubernetesClustersClient;

  constructor(transport: GrpcWebFetchTransport) {
    this.client = new KubernetesClustersClient(transport);
  }

  public createCluster(
    data: CreateKubernetesClusterInput,
  ): Promise<KubernetesCluster> {
    return this.client
      .createCluster({
        name: data.name,
        description: data.description,
        kubeconfig: data.kubeconfig,
      })
      .response.then(({ cluster }) => {
        if (!cluster) {
          return Promise.reject(new Error('missing cluster in response'));
        }
        return fromRpcCluster(cluster);
      });
  }

  public deleteCluster(clusterId: string): Promise<void> {
    return this.client.deleteCluster({ clusterId }).response.then(() => {});
  }

  public getCluster(clusterId: string): Promise<KubernetesCluster> {
    return this.client
      .getCluster({ clusterId })
      .response.then(({ cluster }) => {
        if (!cluster) {
          return Promise.reject(new Error('missing cluster in response'));
        }
        return fromRpcCluster(cluster);
      });
  }

  public listClusters(): Promise<KubernetesCluster[]> {
    return this.client
      .listClusters({})
      .response.then(({ clusters }) => clusters.map(fromRpcCluster));
  }

  public updateCluster(
    data: UpdateKubernetesClusterInput,
  ): Promise<KubernetesCluster> {
    return this.client
      .updateCluster({
        clusterId: data.clusterId,
        name: data.name,
        description: data.description,
        kubeconfig: data.kubeconfig,
      })
      .response.then(({ cluster }) => {
        if (!cluster) {
          return Promise.reject(new Error('missing cluster in response'));
        }
        return fromRpcCluster(cluster);
      });
  }
}

function fromRpcCluster(cluster: KubernetesClusterProto): KubernetesCluster {
  const statusMap: Record<string, KubernetesClusterHealthStatus> = {
    healthy: KubernetesClusterHealthStatus.Healthy,
    unreachable: KubernetesClusterHealthStatus.Unreachable,
  };

  if (!cluster.createdAt) {
    throw new Error('createdAt missing in cluster response');
  }
  if (!cluster.updatedAt) {
    throw new Error('updatedAt missing in cluster response');
  }

  const resolvedHealthStatus = statusMap[cluster.healthStatus];
  if (resolvedHealthStatus === undefined) {
    console.warn(
      `Unknown cluster health status: "${cluster.healthStatus}", falling back to Unreachable`,
    );
  }

  return {
    id: cluster.id,
    name: cluster.name,
    description: cluster.description,
    apiServerUrl: cluster.apiServerUrl,
    caFingerprint: cluster.caFingerprint,
    kubernetesVersion: cluster.kubernetesVersion,
    platform: cluster.platform,
    healthStatus:
      resolvedHealthStatus ?? KubernetesClusterHealthStatus.Unreachable,
    lastHealthCheckAt: cluster.lastHealthCheckAt
      ? new Date(Number(cluster.lastHealthCheckAt.seconds) * 1000).toISOString()
      : undefined,
    createdAt: new Date(Number(cluster.createdAt.seconds) * 1000).toISOString(),
    updatedAt: new Date(Number(cluster.updatedAt.seconds) * 1000).toISOString(),
  };
}
