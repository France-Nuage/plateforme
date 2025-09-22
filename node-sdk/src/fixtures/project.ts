import { faker } from '@faker-js/faker';

import { Project } from '../types';

export const acmeProject: Project = {
  id: '00000000-0000-0000-0000-000000000000',
  name: 'Missile Guiding System',
  organizationId: '00000000-0000-0000-0000-000000000000',
};

export const project = (): Project => ({
  id: faker.string.uuid(),
  name: faker.commerce.productName(),
  organizationId: faker.string.uuid(),
});

export const projects = (count: number): Project[] =>
  [...Array(count)].map(project);
