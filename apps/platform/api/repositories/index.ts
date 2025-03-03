// resource
export { OrganizationRepository } from "./resource/OrganizationRepository";
import { organizationsRepository } from "@france-nuage/api";
export { ProjectRepository } from "./resource/ProjectRepository";
export { FolderRepository } from "./resource/FolderRepository";

// iam
export { IAMMemberRepository } from "./iam/MemberRepository";
export { IAMPolicyRepository } from "./iam/PolicyRepository";
export { RoleRepository } from "./iam/RoleRepository";
export { PermissionRepository } from "./iam/PermissionRepository";

// service
export { ServiceRepository } from "./service/ServiceRepository";

// account billing
export { BillingAccountRepository } from "./billing/BillingAccountRepository";
export { PricingRepository } from "./billing/PricingRepository";

// member
export { MemberRepository } from "./member/MemberRepository";

// compute
export { RegionRepository } from "./compute/RegionRepository";
export { ZoneRepository } from "./compute/ZoneRepository";
export { InstanceRepository } from "./compute/InstanceRepository";

// payment
export { PaymentMethodRepository } from "./payment/PaymentMethodRepository";
