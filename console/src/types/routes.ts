export enum Routes {
  CreateInstance = '/compute/instances/create',
  Home = '/',
  Instances = '/compute/instances',
  Login = '/login',
  ManagedServices = '/managed-services',
  ManagedServiceDetail = '/managed-services/:slug',
  ManagedServiceDeploy = '/managed-services/:slug/deploy',
  ManagedInstances = '/managed-services/instances',
  ManagedInstanceDetail = '/managed-services/instances/:instanceId',
  KubernetesClusters = '/admin/kubernetes-clusters',
  KubernetesClusterCreate = '/admin/kubernetes-clusters/create',
  KubernetesClusterEdit = '/admin/kubernetes-clusters/:clusterId/edit',
}
