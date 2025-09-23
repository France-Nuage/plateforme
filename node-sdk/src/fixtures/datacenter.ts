import { faker } from '@faker-js/faker';

import { Datacenter } from '../models';

export const acmeDatacenter: Datacenter = {
  id: '00000000-0000-0000-0000-000000000000',
  name: 'dc-acme',
};

export const datacenter = (): Datacenter => ({
  id: faker.string.uuid(),
  name: `DC ${faker.company.name()}`,
});

export const datacenters = (count: number): Datacenter[] =>
  [...Array(count)].map(datacenter);
