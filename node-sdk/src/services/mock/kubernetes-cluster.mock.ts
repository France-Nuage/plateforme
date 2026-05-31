import { kubernetesCluster, kubernetesClusters } from '../../fixtures';
import {
  CreateKubernetesClusterInput,
  KubernetesClusterHealthStatus,
  UpdateKubernetesClusterInput,
} from '../../models';
import { KubernetesClusterService } from '../api';

export class KubernetesClusterMockService implements KubernetesClusterService {
  createCluster(data: CreateKubernetesClusterInput) {
    return Promise.resolve(
      kubernetesCluster({
        name: data.name,
        description: data.description,
        healthStatus: KubernetesClusterHealthStatus.Healthy,
      }),
    );
  }

  deleteCluster() {
    return Promise.resolve();
  }

  getCluster(clusterId: string) {
    return Promise.resolve(kubernetesCluster({ id: clusterId }));
  }

  listClusters() {
    return Promise.resolve(kubernetesClusters(3));
  }

  updateCluster(data: UpdateKubernetesClusterInput) {
    return Promise.resolve(
      kubernetesCluster({
        id: data.clusterId,
        name: data.name,
        description: data.description,
      }),
    );
  }
}

export const kubernetesClusterMockService = new KubernetesClusterMockService();
