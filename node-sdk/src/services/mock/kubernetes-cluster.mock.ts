import {
  kubernetesCluster,
  kubernetesClusters,
  kubernetesLabel,
  kubernetesLabels,
  managedServiceRef,
} from '../../fixtures';
import {
  CreateKubernetesClusterInput,
  CreateKubernetesLabelInput,
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

  listLabels() {
    return Promise.resolve(kubernetesLabels(5));
  }

  createLabel(data: CreateKubernetesLabelInput) {
    return Promise.resolve(
      kubernetesLabel({ key: data.key, value: data.value }),
    );
  }

  deleteLabel() {
    return Promise.resolve();
  }

  attachClusterLabel() {
    return Promise.resolve();
  }

  detachClusterLabel() {
    return Promise.resolve();
  }

  listServicesReferencingLabel() {
    return Promise.resolve([managedServiceRef()]);
  }
}

export const kubernetesClusterMockService = new KubernetesClusterMockService();
