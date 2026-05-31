import {
  CreateKubernetesClusterInput,
  KubernetesCluster,
  UpdateKubernetesClusterInput,
} from '../../models';

export interface KubernetesClusterService {
  listClusters: () => Promise<KubernetesCluster[]>;
  getCluster: (clusterId: string) => Promise<KubernetesCluster>;
  createCluster: (
    data: CreateKubernetesClusterInput,
  ) => Promise<KubernetesCluster>;
  updateCluster: (
    data: UpdateKubernetesClusterInput,
  ) => Promise<KubernetesCluster>;
  deleteCluster: (clusterId: string) => Promise<void>;
}
