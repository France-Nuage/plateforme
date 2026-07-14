import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import {
  BillingService,
  HypervisorService,
  InstanceService,
  InvitationService,
  KubernetesClusterService,
  ManagedServiceService,
  OrganizationService,
  ProfileService,
  ProjectService,
  ZeroTrustNetworkService,
  ZeroTrustNetworkTypeService,
  ZoneService,
} from './api';
import {
  hypervisorMockService,
  instanceMockService,
  invitationMockService,
  kubernetesClusterMockService,
  managedServiceMockService,
  organizationMockService,
  profileMockService,
  projectMockService,
  zeroTrustNetworkMockService,
  zeroTrustNetworkTypeMockService,
  zoneMockService,
} from './mock';
import {
  BillingRpcService,
  HypervisorRpcService,
  InstanceRpcService,
  InvitationRpcService,
  KubernetesClusterRpcService,
  ManagedServiceRpcService,
  OrganizationRpcService,
  ProfileRpcService,
  ProjectRpcService,
  ZeroTrustNetworkRpcService,
  ZeroTrustNetworkTypeRpcService,
  ZoneRpcService,
} from './rpc';
import { ServiceMode } from './service-mode';

export type Services = {
  billing: BillingService;
  zone: ZoneService;
  hypervisor: HypervisorService;
  instance: InstanceService;
  invitation: InvitationService;
  kubernetesCluster: KubernetesClusterService;
  managedService: ManagedServiceService;
  organization: OrganizationService;
  profile: ProfileService;
  project: ProjectService;
  zeroTrustNetwork: ZeroTrustNetworkService;
  zeroTrustNetworkType: ZeroTrustNetworkTypeService;
};

/**
 * Configures service resolver with transport-based service instances.
 *
 * Creates service implementations for both Mock and RPC modes, where RPC services
 * are instantiated with the provided transport for proper authentication and error handling.
 *
 * @param transport - The configured gRPC transport instance
 * @returns Service resolver mapping for each ServiceMode
 */
export function configureResolver(
  transport: GrpcWebFetchTransport,
): Record<ServiceMode, Services> {
  return {
    [ServiceMode.Mock]: {
      billing: new BillingRpcService(transport),
      hypervisor: hypervisorMockService,
      instance: instanceMockService,
      invitation: invitationMockService,
      kubernetesCluster: kubernetesClusterMockService,
      managedService: managedServiceMockService,
      organization: organizationMockService,
      profile: profileMockService,
      project: projectMockService,
      zeroTrustNetwork: zeroTrustNetworkMockService,
      zeroTrustNetworkType: zeroTrustNetworkTypeMockService,
      zone: zoneMockService,
    },
    [ServiceMode.Rpc]: {
      billing: new BillingRpcService(transport),
      hypervisor: new HypervisorRpcService(transport),
      instance: new InstanceRpcService(transport),
      invitation: new InvitationRpcService(transport),
      kubernetesCluster: new KubernetesClusterRpcService(transport),
      managedService: new ManagedServiceRpcService(transport),
      organization: new OrganizationRpcService(transport),
      profile: new ProfileRpcService(transport),
      project: new ProjectRpcService(transport),
      zeroTrustNetwork: new ZeroTrustNetworkRpcService(transport),
      zeroTrustNetworkType: new ZeroTrustNetworkTypeRpcService(transport),
      zone: new ZoneRpcService(transport),
    },
  };
}
