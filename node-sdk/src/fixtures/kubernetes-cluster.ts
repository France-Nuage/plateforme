import { faker } from '@faker-js/faker';

import {
  KubernetesCluster,
  KubernetesClusterHealthStatus,
  KubernetesLabel,
  ManagedServiceRef,
} from '../models';

export const kubernetesLabel = (
  preset?: Partial<KubernetesLabel>,
): KubernetesLabel => ({
  id: faker.string.uuid(),
  key: faker.helpers.arrayElement(['availability', 'region', 'tier']),
  value: faker.helpers.arrayElement(['ft', 'best-effort', 'eu', 'us']),
  system: false,
  createdAt: faker.date.recent().toISOString(),
  updatedAt: faker.date.recent().toISOString(),
  ...preset,
});

export const kubernetesLabels = (
  count: number,
  preset?: Partial<KubernetesLabel>,
): KubernetesLabel[] => [...Array(count)].map(() => kubernetesLabel(preset));

export const managedServiceRef = (
  preset?: Partial<ManagedServiceRef>,
): ManagedServiceRef => ({
  id: faker.string.uuid(),
  slug: faker.helpers.slugify(faker.commerce.productName()).toLowerCase(),
  name: faker.commerce.productName(),
  ...preset,
});

export const kubernetesCluster = (
  preset?: Partial<KubernetesCluster>,
): KubernetesCluster => ({
  id: faker.string.uuid(),
  name: faker.helpers.arrayElement(['prod-eu', 'prod-us', 'staging-eu', 'dev']),
  description: faker.datatype.boolean() ? faker.lorem.sentence() : undefined,
  apiServerUrl: `https://${faker.internet.domainName()}:6443`,
  caFingerprint: faker.datatype.boolean()
    ? faker.string.hexadecimal({ length: 64, prefix: '' })
    : undefined,
  kubernetesVersion: faker.helpers.arrayElement([
    'v1.30.4',
    'v1.31.1',
    'v1.32.2',
  ]),
  platform: faker.helpers.arrayElement(['linux/amd64', 'linux/arm64']),
  healthStatus: faker.helpers.arrayElement(
    Object.values(KubernetesClusterHealthStatus),
  ),
  lastHealthCheckAt: faker.datatype.boolean()
    ? faker.date.recent().toISOString()
    : undefined,
  createdAt: faker.date.recent().toISOString(),
  updatedAt: faker.date.recent().toISOString(),
  labels: kubernetesLabels(faker.number.int({ min: 0, max: 3 })),
  ...preset,
});

export const kubernetesClusters = (
  count: number,
  preset?: Partial<KubernetesCluster>,
): KubernetesCluster[] =>
  [...Array(count)].map(() => kubernetesCluster(preset));
