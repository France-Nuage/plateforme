import {
  CreateKubernetesClusterInput,
  CreateKubernetesLabelInput,
  KubernetesCluster,
  KubernetesLabel,
  ManagedServiceRef,
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
  listLabels: () => Promise<KubernetesLabel[]>;
  createLabel: (data: CreateKubernetesLabelInput) => Promise<KubernetesLabel>;
  deleteLabel: (labelId: string) => Promise<void>;
  attachClusterLabel: (clusterId: string, labelId: string) => Promise<void>;
  detachClusterLabel: (clusterId: string, labelId: string) => Promise<void>;
  listServicesReferencingLabel: (
    labelId: string,
  ) => Promise<ManagedServiceRef[]>;
}
