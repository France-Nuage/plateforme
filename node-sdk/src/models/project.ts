export type Project = {
  /**
   * The project slug (primary key).
   */
  slug: string;

  /**
   * The project name.
   */
  name: string;

  /**
   * The organization slug this project belongs to.
   */
  organizationSlug: string;
};

/**
 * The project form creation/update value.
 */
export type ProjectFormValue = Pick<Project, 'name' | 'organizationSlug'>;
