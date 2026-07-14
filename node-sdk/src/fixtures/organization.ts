import { faker } from '@faker-js/faker';

import { Organization } from '../models';

export const acmeOrganization: Organization = {
  name: 'ACME',
  slug: 'acme',
};

export const organization = (): Organization => {
  const name = faker.company.name();
  return {
    name,
    slug: name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, ''),
  };
};

export const organizations = (count: number): Organization[] =>
  [...Array(count)].map(organization);
