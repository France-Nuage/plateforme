import { faker } from '@faker-js/faker';

import { Hypervisor } from '@/types';

export const hypervisor = (): Hypervisor => ({
  datacenterId: faker.string.uuid(),
  id: faker.string.uuid(),
  organizationId: faker.string.uuid(),
  storageName: faker.commerce.productName(),
  url: faker.internet.url(),
});

export const hypervisors = (count: number): Hypervisor[] =>
  [...Array(count)].map(hypervisor);
