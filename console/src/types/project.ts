export type Project = {
  /**
   * The project id.
   */
  id: string;

  /**
   * The project name.
   */
  name: string;

  /**
   * The project organization id.
   */
  organizationId: string;
};

/**
 * The project form creation/update value.
 */
export type ProjectFormValue = Pick<Project, 'name' | 'organizationId'>;
