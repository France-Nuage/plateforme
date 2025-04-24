import { faker } from "@faker-js/faker";
import { InstanceInfo, InstanceStatus } from "../protocol/instances";

// @ts-expect-error allows to serialize a big int to a string
BigInt.prototype.toJSON = function () {
  return this.toString();
};

export const instance = (): InstanceInfo => ({
  id: faker.string.uuid(),
  status: faker.helpers.arrayElement([
    InstanceStatus.RUNNING,
    InstanceStatus.STOPPED,
  ]),
  maxCpuCores: faker.helpers.arrayElement([1, 2, 4, 8]),
  cpuUsagePercent: faker.number.float({ min: 0, max: 100, fractionDigits: 2 }),
  maxMemoryBytes: BigInt(
    faker.number.int({ min: 1073741824, max: 68719476736 }),
  ),
  memoryUsageBytes: BigInt(
    faker.number.int({ min: 1073741824, max: 68719476736 }),
  ),
  name: faker.commerce.productName(),
});

export const instances = (count: number): InstanceInfo[] =>
  [...Array(count)].map(instance);
