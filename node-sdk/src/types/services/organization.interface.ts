import { Organization, OrganizationFormValue } from '../models';

/**
 * Define organization actions.
 */
export interface OrganizationService {
  /** List the available organizations */
  list: () => Promise<Organization[]>;

  /** Create a new organization */
  create: (data: OrganizationFormValue) => Promise<Organization>;
}
