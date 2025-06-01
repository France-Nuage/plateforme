import { Project, ProjectFormValue } from "@/types";

/**
 * Define project actions.
 */
export interface ProjectService {
  /** List the available projects */
  list: () => Promise<Project[]>;

  /** Create a new project */
  create: (data: ProjectFormValue) => Promise<Project>;
}
