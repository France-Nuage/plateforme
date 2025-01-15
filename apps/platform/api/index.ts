import {SecurityRepository} from "./security";
import {
    BillingAccountRepository,
    FolderRepository,
    IAMMemberRepository,
    IAMPolicyRepository,
    InstanceRepository,
    MemberRepository,
    OrganizationRepository,
    PaymentMethodRepository,
    PermissionRepository,
    PricingRepository,
    ProjectRepository,
    RegionRepository,
    RoleRepository,
    ServiceRepository,
    ZoneRepository,
} from "./repositories";
import type {AxiosInstance} from "axios";

const repositories: any = (
  client: AxiosInstance,
  config: Record<any, any>,
) => ({
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
  organizations: OrganizationRepository(client, config),
  projects: ProjectRepository(client, config),
  services: ServiceRepository(client, config),
  folders: FolderRepository(client, config),
  roles: RoleRepository(client, config),
  permissions: PermissionRepository(client, config),
});

export default repositories;
