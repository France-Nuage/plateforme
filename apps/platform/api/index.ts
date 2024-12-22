import { SecurityRepository } from './security';
import {
  OrganizationRepository,
  ProjectRepository,
  IAMMemberRepository,
  IAMPolicyRepository,
  MemberRepository,
  ServiceRepository,
  FolderRepository,
  AccountBillingRepository,
  RoleRepository,
  PermissionRepository,
  RegionRepository,
  ZoneRepository
} from './repositories';
import type { AxiosInstance } from 'axios';

const repositories: any = (client: AxiosInstance, config: Record<any, any>) => ({
  iam: {
    members: IAMMemberRepository(client, config),
    policies: IAMPolicyRepository(client, config),
  },
  compute: {
    regions: RegionRepository(client, config),
    zones: ZoneRepository(client, config),
  },
  members: MemberRepository(client, config),
  security: SecurityRepository(client, config),
  organizations: OrganizationRepository(client, config),
  projects: ProjectRepository(client, config),
  services: ServiceRepository(client, config),
  folders: FolderRepository(client, config),
  accountBillings: AccountBillingRepository(client, config),
  roles: RoleRepository(client, config),
  permissions: PermissionRepository(client, config),
});

export default repositories;
