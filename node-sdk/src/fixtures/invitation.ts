import { faker } from '@faker-js/faker';

import { Invitation } from '../models';

export const invitation = (): Invitation => ({
  id: faker.string.uuid(),
  organizationSlug: faker.helpers.arrayElement(['acme', 'corp', 'dev']),
  userId: faker.string.uuid(),
});

export const invitations = (count: number): Invitation[] =>
  [...Array(count)].map(invitation);
