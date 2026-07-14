import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import {
  KubernetesClusterProto,
  KubernetesLabelProto,
} from '../../generated/rpc/kubernetes';
import { KubernetesClustersClient } from '../../generated/rpc/kubernetes.client';
import {
  CreateKubernetesClusterInput,
  CreateKubernetesLabelInput,
  KubernetesCluster,
  KubernetesClusterHealthStatus,
  KubernetesLabel,
  ManagedServiceRef,
  UpdateKubernetesClusterInput,
} from '../../models';
import { KubernetesClusterService } from '../api';

export class KubernetesClusterRpcService implements KubernetesClusterService {
  private client: KubernetesClustersClient;

  constructor(transport: GrpcWebFetchTransport) {
    this.client = new KubernetesClustersClient(transport);
  }

  public attachClusterLabel(clusterId: string, labelId: string): Promise<void> {
    return this.client
      .attachClusterLabel({ clusterId, labelId })
      .response.then(() => {});
  }

  public createCluster(
    data: CreateKubernetesClusterInput,
  ): Promise<KubernetesCluster> {
    return this.client
      .createCluster({
        name: data.name,
        description: data.description,
        kubeconfig: data.kubeconfig,
        labelIds: data.labelIds ?? [],
      })
      .response.then(({ cluster }) => {
        if (!cluster) {
          return Promise.reject(new Error('missing cluster in response'));
        }
        return fromRpcCluster(cluster);
      });
  }

  public createLabel(
    data: CreateKubernetesLabelInput,
  ): Promise<KubernetesLabel> {
    return this.client
      .createLabel({ key: data.key, value: data.value })
      .response.then(({ label }) => {
        if (!label) {
          return Promise.reject(new Error('missing label in response'));
        }
        return fromRpcLabel(label);
      });
  }

  public deleteCluster(clusterId: string): Promise<void> {
    return this.client.deleteCluster({ clusterId }).response.then(() => {});
  }

  public deleteLabel(labelId: string): Promise<void> {
    return this.client.deleteLabel({ labelId }).response.then(() => {});
  }

  public detachClusterLabel(clusterId: string, labelId: string): Promise<void> {
    return this.client
      .detachClusterLabel({ clusterId, labelId })
      .response.then(() => {});
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

  public listLabels(): Promise<KubernetesLabel[]> {
    return this.client
      .listLabels({})
      .response.then(({ labels }) => labels.map(fromRpcLabel));
  }

  public listServicesReferencingLabel(
    labelId: string,
  ): Promise<ManagedServiceRef[]> {
    return this.client
      .listServicesReferencingLabel({ labelId })
      .response.then(({ services }) =>
        services.map((service) => ({
          id: service.id,
          slug: service.slug,
          name: service.name,
        })),
      );
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
    labels: cluster.labels.map(fromRpcLabel),
  };
}

function fromRpcLabel(label: KubernetesLabelProto): KubernetesLabel {
  if (!label.createdAt) {
    throw new Error('createdAt missing in label response');
  }
  if (!label.updatedAt) {
    throw new Error('updatedAt missing in label response');
  }

  return {
    id: label.id,
    key: label.key,
    value: label.value,
    system: label.system,
    createdAt: new Date(Number(label.createdAt.seconds) * 1000).toISOString(),
    updatedAt: new Date(Number(label.updatedAt.seconds) * 1000).toISOString(),
  };
}
