import { faker } from '@faker-js/faker';

import { Instance, InstanceStatus } from '../models';

export const instance = (preset?: Partial<Instance>): Instance => ({
  cpuUsagePercent: faker.number.float({ fractionDigits: 2, max: 100, min: 0 }),
  createdAt: faker.date.recent().toISOString(),
  diskUsageBytes: faker.number.int({ max: 1073741824000, min: 10737418240 }),
  hypervisorId: faker.string.uuid(),
  id: faker.string.uuid(),
  ipV4: faker.internet.ipv4(),
  maxCpuCores: faker.helpers.arrayElement([1, 2, 4, 8]),
  maxDiskBytes: faker.number.int({ max: 1073741824000, min: 10737418240 }),
  maxMemoryBytes: faker.number.int({ max: 68719476736, min: 1073741824 }),
  memoryUsageBytes: faker.number.int({ max: 68719476736, min: 1073741824 }),
  name: faker.commerce.productName(),
  projectId: faker.string.uuid(),
  status: faker.helpers.arrayElement(Object.values(InstanceStatus)),
  updatedAt: faker.date.recent().toISOString(),
  zeroTrustNetworkId: faker.string.uuid(),
  ...preset,
});

export const instances = (
  count: number,
  preset?: Partial<Instance>,
): Instance[] => [...Array(count)].map(() => instance(preset));
