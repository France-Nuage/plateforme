import { faker } from '@faker-js/faker';

import { Project } from '../models';

export const acmeProject: Project = {
  slug: 'unattributed',
  name: 'Missile Guiding System',
  organizationSlug: 'acme',
};

export const project = (): Project => ({
  slug: faker.helpers.arrayElement(['unattributed', 'default', 'production']),
  name: faker.commerce.productName(),
  organizationSlug: faker.helpers.arrayElement(['acme', 'corp', 'dev']),
});

export const projects = (count: number): Project[] =>
  [...Array(count)].map(project);
