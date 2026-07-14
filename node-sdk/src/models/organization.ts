/**
 * Represent an organization.
 */
export type Organization = {
  /**
   * The organization slug (primary key, DNS-compatible identifier).
   */
  slug: string;

  /**
   * The organization name.
   */
  name: string;

  /**
   * The parent organization slug, if any.
   */
  parentSlug?: string;
};

/**
 * The organization form creation/update value.
 */
export type OrganizationFormValue = Pick<Organization, 'name' | 'parentSlug'>;
