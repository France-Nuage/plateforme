import { faker } from '@faker-js/faker';

import { Organization } from '../models';

export const acmeOrganization: Organization = {
  id: '00000000-0000-0000-0000-000000000000',
  name: 'ACME',
  slug: 'acme',
};

export const organization = (): Organization => {
  const name = faker.company.name();
  return {
    id: faker.string.uuid(),
    name,
    slug: name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, ''),
  };
};

export const organizations = (count: number): Organization[] =>
  [...Array(count)].map(organization);
