import { faker } from '@faker-js/faker';

import { Hypervisor } from '../types';
import { acmeDatacenter } from './datacenter';
import { acmeOrganization } from './organization';

export const acmeHypervisor: Hypervisor = {
  datacenterId: acmeDatacenter.id,
  id: '00000000-0000-0000-0000-000000000000',
  organizationId: acmeOrganization.id,
  storageName: 'local-lvm',
  url: 'https://hypervisor.acme',
};

export const hypervisor = (preset?: Partial<Hypervisor>): Hypervisor => ({
  datacenterId: faker.string.uuid(),
  id: faker.string.uuid(),
  organizationId: faker.string.uuid(),
  storageName: faker.commerce.productName(),
  url: faker.internet.url(),
  ...preset,
});

export const hypervisors = (
  count: number,
  preset?: Partial<Hypervisor>,
): Hypervisor[] => [...Array(count)].map(() => hypervisor(preset));
