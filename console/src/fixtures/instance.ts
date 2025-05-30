import { faker } from '@faker-js/faker';

import { Instance } from '@/types';
import { InstanceStatus } from '@/types';

export const instance = (): Instance => ({
  cpuUsagePercent: faker.number.float({ fractionDigits: 2, max: 100, min: 0 }),
  id: faker.string.uuid(),
  maxCpuCores: faker.helpers.arrayElement([1, 2, 4, 8]),
  maxMemoryBytes: faker.number.int({ max: 68719476736, min: 1073741824 }),
  memoryUsageBytes: faker.number.int({ max: 68719476736, min: 1073741824 }),
  name: faker.commerce.productName(),
  status: faker.helpers.arrayElement(Object.values(InstanceStatus)),
});

export const instances = (count: number): Instance[] =>
  [...Array(count)].map(instance);
