import { ServiceMode } from '@/types';

import { DatacenterService } from './datacenter.interface';
import { datacenterMockService } from './datacenter.mock';
import { datacenterRpcService } from './datacenter.rpc';
import { HypervisorService } from './hypervisor.interface';
import { hypervisorMockService } from './hypervisor.mock';
import { hypervisorsRpcService } from './hypervisor.rpc';
import { InstanceService } from './instance.interface';
import { instanceMockService } from './instance.mock';
import { instanceRpcService } from './instance.rpc';
import { OrganizationService } from './organization.interface';
import { organizationMockService } from './organization.mock';
import { organizationRpcService } from './organization.rpc';
import { ProjectService } from './project.interface';
import { projectMockService } from './project.mock';
import { projectRpcService } from './project.rpc';
import { ZeroTrustNetworkTypeService } from './zero-trust-network-type.interface';
import { zeroTrustNetworkTypeMockService } from './zero-trust-network-type.mock';
import { zeroTrustNetworkTypeRpcService } from './zero-trust-network-type.rpc';
import { ZeroTrustNetworkService } from './zero-trust-network.interface';
import { zeroTrustNetworkMockService } from './zero-trust-network.mock';
import { zeroTrustNetworkRpcService } from './zero-trust-network.rpc';

type Services = {
  datacenter: DatacenterService;
  hypervisor: HypervisorService;
  instance: InstanceService;
  organization: OrganizationService;
  project: ProjectService;
  zeroTrustNetwork: ZeroTrustNetworkService;
  zeroTrustNetworkType: ZeroTrustNetworkTypeService;
};

export const services: Record<ServiceMode, Services> = {
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
    datacenter: datacenterRpcService,
    hypervisor: hypervisorsRpcService,
    instance: instanceRpcService,
    organization: organizationRpcService,
    project: projectRpcService,
    zeroTrustNetwork: zeroTrustNetworkRpcService,
    zeroTrustNetworkType: zeroTrustNetworkTypeRpcService,
  },
};
