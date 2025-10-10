import { faker } from '@faker-js/faker';

import { Hypervisor } from '../models';
import { acmeOrganization } from './organization';
import { acmeZone } from './zone';

export const acmeHypervisor: Hypervisor = {
  id: '00000000-0000-0000-0000-000000000000',
  organizationId: acmeOrganization.id,
  storageName: 'local-lvm',
  url: 'https://hypervisor.acme',
  zoneId: acmeZone.id,
};

export const hypervisor = (preset?: Partial<Hypervisor>): Hypervisor => ({
  id: faker.string.uuid(),
  organizationId: faker.string.uuid(),
  storageName: faker.commerce.productName(),
  url: faker.internet.url(),
  zoneId: faker.string.uuid(),
  ...preset,
});

export const hypervisors = (
  count: number,
  preset?: Partial<Hypervisor>,
): Hypervisor[] => [...Array(count)].map(() => hypervisor(preset));
