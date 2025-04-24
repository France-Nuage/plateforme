import { Instance, InstanceStatus } from "@/types";
import { faker } from "@faker-js/faker";

export const instance = (): Instance => ({
  id: faker.string.uuid(),
  status: faker.helpers.arrayElement(Object.values(InstanceStatus)),
  maxCpuCores: faker.helpers.arrayElement([1, 2, 4, 8]),
  cpuUsagePercent: faker.number.float({ min: 0, max: 100, fractionDigits: 2 }),
  maxMemoryBytes: faker.number
    .int({ min: 1073741824, max: 68719476736 })
    .toString(),
  memoryUsageBytes: faker.number
    .int({ min: 1073741824, max: 68719476736 })
    .toString(),
  name: faker.commerce.productName(),
});

export const instances = (count: number): Instance[] =>
  [...Array(count)].map(instance);
