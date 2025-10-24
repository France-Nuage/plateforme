import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import {
  HypervisorService,
  InstanceService,
  InvitationService,
  OrganizationService,
  ProjectService,
  ZeroTrustNetworkService,
  ZeroTrustNetworkTypeService,
  ZoneService,
} from './api';
import {
  hypervisorMockService,
  instanceMockService,
  invitationMockService,
  organizationMockService,
  projectMockService,
  zeroTrustNetworkMockService,
  zeroTrustNetworkTypeMockService,
  zoneMockService,
} from './mock';
import {
  HypervisorRpcService,
  InstanceRpcService,
  InvitationRpcService,
  OrganizationRpcService,
  ProjectRpcService,
  ZeroTrustNetworkRpcService,
  ZeroTrustNetworkTypeRpcService,
  ZoneRpcService,
} from './rpc';
import { ServiceMode } from './service-mode';

export type Services = {
  zone: ZoneService;
  hypervisor: HypervisorService;
  instance: InstanceService;
  invitation: InvitationService;
  organization: OrganizationService;
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
      hypervisor: hypervisorMockService,
      instance: instanceMockService,
      invitation: invitationMockService,
      organization: organizationMockService,
      project: projectMockService,
      zeroTrustNetwork: zeroTrustNetworkMockService,
      zeroTrustNetworkType: zeroTrustNetworkTypeMockService,
      zone: zoneMockService,
    },
    [ServiceMode.Rpc]: {
      hypervisor: new HypervisorRpcService(transport),
      instance: new InstanceRpcService(transport),
      invitation: new InvitationRpcService(transport),
      organization: new OrganizationRpcService(transport),
      project: new ProjectRpcService(transport),
      zeroTrustNetwork: new ZeroTrustNetworkRpcService(transport),
      zeroTrustNetworkType: new ZeroTrustNetworkTypeRpcService(transport),
      zone: new ZoneRpcService(transport),
    },
  };
}
