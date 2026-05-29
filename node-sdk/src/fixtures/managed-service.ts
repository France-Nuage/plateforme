import { faker } from '@faker-js/faker';

import {
  ManagedInstanceStatus,
  ManagedService,
  ManagedServiceInstance,
  ManagedServiceVersion,
} from '../models';

export const managedService = (
  preset?: Partial<ManagedService>,
): ManagedService => ({
  id: faker.string.uuid(),
  slug: faker.helpers.arrayElement([
    'vaultwarden',
    'nextcloud',
    'n8n',
    'metabase',
  ]),
  name: faker.helpers.arrayElement([
    'Vaultwarden',
    'Nextcloud',
    'n8n',
    'Metabase',
  ]),
  description: faker.lorem.paragraph(),
  category: faker.helpers.arrayElement([
    'security',
    'collaboration',
    'analytics',
    'automation',
  ]),
  databaseEngine: faker.helpers.arrayElement(['cnpg', 'mariadb']),
  iconUrl: undefined,
  createdAt: faker.date.recent().toISOString(),
  ...preset,
});

export const managedServices = (
  count: number,
  preset?: Partial<ManagedService>,
): ManagedService[] => [...Array(count)].map(() => managedService(preset));

export const managedServiceVersion = (
  preset?: Partial<ManagedServiceVersion>,
): ManagedServiceVersion => ({
  id: faker.string.uuid(),
  serviceId: faker.string.uuid(),
  chartVersion: `${faker.number.int({ min: 1, max: 5 })}.${faker.number.int({ min: 0, max: 20 })}.${faker.number.int({ min: 0, max: 10 })}`,
  appVersion: `${faker.number.int({ min: 1, max: 5 })}.${faker.number.int({ min: 0, max: 20 })}.${faker.number.int({ min: 0, max: 10 })}`,
  ociReference: `oci://registry.gitlab.com/france-nuage/charts/${faker.helpers.arrayElement(['vaultwarden', 'nextcloud', 'n8n'])}`,
  configurableValuesSchema: undefined,
  createdAt: faker.date.recent().toISOString(),
  ...preset,
});

export const managedServiceInstance = (
  preset?: Partial<ManagedServiceInstance>,
): ManagedServiceInstance => ({
  id: faker.string.uuid(),
  serviceId: faker.string.uuid(),
  versionId: faker.string.uuid(),
  projectId: faker.string.uuid(),
  organizationId: faker.string.uuid(),
  namespace: `managed-${faker.string.alphanumeric(8)}`,
  releaseName: `${faker.helpers.arrayElement(['vaultwarden', 'nextcloud', 'n8n'])}-${faker.string.alphanumeric(6)}`,
  userValues: undefined,
  status: faker.helpers.arrayElement(Object.values(ManagedInstanceStatus)),
  createdAt: faker.date.recent().toISOString(),
  ...preset,
});

export const managedServiceInstances = (
  count: number,
  preset?: Partial<ManagedServiceInstance>,
): ManagedServiceInstance[] =>
  [...Array(count)].map(() => managedServiceInstance(preset));
