import { ServiceMode } from '@/types';

import { HypervisorService } from './hypervisor.interface';
import { hypervisorMockService } from './hypervisor.mock';
import { hypervisorsRpcService } from './hypervisor.rpc';
import { InstanceService } from './instance.interface';
import { instanceMockService } from './instance.mock';
import { instanceRpcService } from './instance.rpc';
import { OrganizationService } from './organization.interface';
import { ProjectService } from './project.interface';
import { organizationMockService } from './organization.mock';
import { projectMockService } from './project.mock';

type Services = {
  hypervisor: HypervisorService;
  instance: InstanceService;
  organization: OrganizationService;
  project: ProjectService;
};

export const services: Record<ServiceMode, Services> = {
  [ServiceMode.Mock]: {
    hypervisor: hypervisorMockService,
    instance: instanceMockService,
    organization: organizationMockService,
    project: projectMockService,
  },
  [ServiceMode.Rpc]: {
    hypervisor: hypervisorsRpcService,
    instance: instanceRpcService,
    organization: organizationMockService,
    project: projectMockService,
  },
};
