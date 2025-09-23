import { Project, ProjectFormValue } from '../../models';

/**
 * Define project actions.
 */
export interface ProjectService {
  /** List the available projects */
  list: () => Promise<Project[]>;

  /** Create a new project */
  create: (data: ProjectFormValue) => Promise<Project>;
}
