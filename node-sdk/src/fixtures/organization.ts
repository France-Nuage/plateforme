import { faker } from '@faker-js/faker';

import { Organization } from '../models';

export const acmeOrganization: Organization = {
  id: '00000000-0000-0000-0000-000000000000',
  name: 'ACME',
};

export const organization = (): Organization => ({
  id: faker.string.uuid(),
  name: faker.commerce.productName(),
});

export const organizations = (count: number): Organization[] =>
  [...Array(count)].map(organization);
