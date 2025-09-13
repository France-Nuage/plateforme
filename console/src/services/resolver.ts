import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { ServiceMode } from '@/types';

import { DatacenterService } from './datacenter.interface';
import { datacenterMockService } from './datacenter.mock';
import { DatacenterRpcService } from './datacenter.rpc';
import { HypervisorService } from './hypervisor.interface';
import { hypervisorMockService } from './hypervisor.mock';
import { HypervisorRpcService } from './hypervisor.rpc';
import { InstanceService } from './instance.interface';
import { instanceMockService } from './instance.mock';
import { InstanceRpcService } from './instance.rpc';
import { OrganizationService } from './organization.interface';
import { organizationMockService } from './organization.mock';
import { OrganizationRpcService } from './organization.rpc';
import { ProjectService } from './project.interface';
import { projectMockService } from './project.mock';
import { ProjectRpcService } from './project.rpc';
import { ZeroTrustNetworkTypeService } from './zero-trust-network-type.interface';
import { zeroTrustNetworkTypeMockService } from './zero-trust-network-type.mock';
import { ZeroTrustNetworkTypeRpcService } from './zero-trust-network-type.rpc';
import { ZeroTrustNetworkService } from './zero-trust-network.interface';
import { zeroTrustNetworkMockService } from './zero-trust-network.mock';
import { ZeroTrustNetworkRpcService } from './zero-trust-network.rpc';

type Services = {
  datacenter: DatacenterService;
  hypervisor: HypervisorService;
  instance: InstanceService;
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
      datacenter: datacenterMockService,
      hypervisor: hypervisorMockService,
      instance: instanceMockService,
      organization: organizationMockService,
      project: projectMockService,
      zeroTrustNetwork: zeroTrustNetworkMockService,
      zeroTrustNetworkType: zeroTrustNetworkTypeMockService,
    },
    [ServiceMode.Rpc]: {
      datacenter: new DatacenterRpcService(transport),
      hypervisor: new HypervisorRpcService(transport),
      instance: new InstanceRpcService(transport),
      organization: new OrganizationRpcService(transport),
      project: new ProjectRpcService(transport),
      zeroTrustNetwork: new ZeroTrustNetworkRpcService(transport),
      zeroTrustNetworkType: new ZeroTrustNetworkTypeRpcService(transport),
    },
  };
}
