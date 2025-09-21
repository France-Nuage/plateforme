import {
  acmeOrganization,
  organization,
  organizations,
} from '../../fixtures/organization';
import { OrganizationFormValue } from '../../types';
import { OrganizationService } from '../../types';

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
    return Promise.resolve([acmeOrganization, ...organizations(4)]);
  }
}

/**
 * The instance of the organization mock service.
 */
export const organizationMockService = new OrganizationMockService();
