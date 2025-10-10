import { faker } from '@faker-js/faker';

import { Zone } from '../models';

export const acmeZone: Zone = {
  id: '00000000-0000-0000-0000-000000000000',
  name: 'dc-acme',
};

export const zone = (): Zone => ({
  id: faker.string.uuid(),
  name: `DC ${faker.company.name()}`,
});

export const zones = (count: number): Zone[] => [...Array(count)].map(zone);
