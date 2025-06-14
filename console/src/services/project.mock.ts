import { acme, project, projects } from '@/fixtures/project';
import { ProjectFormValue } from '@/types';

import { ProjectService } from './project.interface';

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
    return Promise.resolve([acme, ...projects(3)]);
  }
}

/**
 * The instance of the project mock service.
 */
export const projectMockService = new ProjectMockService();
