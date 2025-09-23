import { acmeProject, project, projects } from '../../fixtures';
import { ProjectFormValue } from '../../models';
import { ProjectService } from '../api';

/**
 * The mock implementation of the project service.
 */
export class ProjectMockService implements ProjectService {
  /** @inheritdoc */
  create(data: ProjectFormValue) {
    return Promise.resolve({ ...project(), ...data });
  }

  /** @inheritdoc */
  list() {
    return Promise.resolve([acmeProject, ...projects(3)]);
  }
}

/**
 * The instance of the project mock service.
 */
export const projectMockService = new ProjectMockService();
