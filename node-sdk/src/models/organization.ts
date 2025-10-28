/**
 * Represent an organization.
 */
export type Organization = {
  /**
   * The organization id.
   */
  id: string;

  /**
   * The organization name.
   */
  name: string;

  /**
   * The organization parent id, if any.
   */
  parentId?: string;
};

/**
 * The organization form creation/update value.
 */
export type OrganizationFormValue = Pick<Organization, 'name' | 'parentId'>;
