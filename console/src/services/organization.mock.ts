import { organization, organizations } from "@/fixtures/organization";
import { OrganizationService } from "./organization.interface";
import { OrganizationFormValue, Organization } from "@/types";

/**
 * The mock implementation of the organization service.
 */
export class OrganizationMockService implements OrganizationService {
  /** @inheritdoc */
  create(data: OrganizationFormValue) {
    return Promise.resolve({ ...organization(), ...data });
  }

  /** @inheritdoc */
  list() {
    return Promise.resolve(organizations(3));
  }
}

/**
 * The instance of the organization mock service.
 */
export const organizationMockService = new OrganizationMockService();
