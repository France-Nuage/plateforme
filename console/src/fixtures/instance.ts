import { faker } from '@faker-js/faker';

import { Instance } from '@/types';
import { InstanceStatus } from '@/types';

export const instance = (): Instance => ({
  id: faker.string.uuid(),
  status: faker.helpers.arrayElement(Object.values(InstanceStatus)),
  maxCpuCores: faker.helpers.arrayElement([1, 2, 4, 8]),
  cpuUsagePercent: faker.number.float({ min: 0, max: 100, fractionDigits: 2 }),
  maxMemoryBytes: faker.number.int({ min: 1073741824, max: 68719476736 }),
  memoryUsageBytes: faker.number.int({ min: 1073741824, max: 68719476736 }),
  name: faker.commerce.productName(),
});

export const instances = (count: number): Instance[] =>
  [...Array(count)].map(instance);
