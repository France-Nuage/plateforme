import { SecurityRepository } from "./security";
import {
  ProjectRepository,
  IAMMemberRepository,
  IAMPolicyRepository,
  MemberRepository,
  ServiceRepository,
  FolderRepository,
  BillingAccountRepository,
  RoleRepository,
  PermissionRepository,
  RegionRepository,
  ZoneRepository,
  InstanceRepository,
  PricingRepository,
  PaymentMethodRepository,
} from "./repositories";
import { organizationsRepository } from "@france-nuage/api";
import type { $Fetch } from "ofetch";

const repositories = (client: $Fetch, config: Record<any, any>) => ({
  iam: {
    members: IAMMemberRepository(client, config),
    policies: IAMPolicyRepository(client, config),
  },
  compute: {
    regions: RegionRepository(client, config),
    zones: ZoneRepository(client, config),
    instances: InstanceRepository(client, config),
  },
  billing: {
    pricing: PricingRepository(client, config),
    accounts: BillingAccountRepository(client, config),
  },
  payment: {
    methods: PaymentMethodRepository(client, config),
  },
  members: MemberRepository(client, config),
  security: SecurityRepository(client, config),
  organizations: organizationsRepository(client, config),
  projects: ProjectRepository(client, config),
  services: ServiceRepository(client, config),
  folders: FolderRepository(client, config),
  roles: RoleRepository(client, config),
  permissions: PermissionRepository(client, config),
});

export default repositories;
