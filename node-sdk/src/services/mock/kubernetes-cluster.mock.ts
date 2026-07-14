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
  attachClusterLabel() {
    return Promise.resolve();
  }

  createCluster(data: CreateKubernetesClusterInput) {
    return Promise.resolve(
      kubernetesCluster({
        name: data.name,
        description: data.description,
        healthStatus: KubernetesClusterHealthStatus.Healthy,
      }),
    );
  }

  createLabel(data: CreateKubernetesLabelInput) {
    return Promise.resolve(
      kubernetesLabel({ key: data.key, value: data.value }),
    );
  }

  deleteCluster() {
    return Promise.resolve();
  }

  deleteLabel() {
    return Promise.resolve();
  }

  detachClusterLabel() {
    return Promise.resolve();
  }

  getCluster(clusterId: string) {
    return Promise.resolve(kubernetesCluster({ id: clusterId }));
  }

  listClusters() {
    return Promise.resolve(kubernetesClusters(3));
  }

  listLabels() {
    return Promise.resolve(kubernetesLabels(5));
  }

  listServicesReferencingLabel() {
    return Promise.resolve([managedServiceRef()]);
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
