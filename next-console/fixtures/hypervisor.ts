import { Hypervisor } from "@/types";
import { faker } from "@faker-js/faker";

export const hypervisor = (): Hypervisor => ({
  id: faker.string.uuid(),
  storageName: faker.commerce.productName(),
  url: faker.internet.url(),
});

export const hypervisors = (count: number): Hypervisor[] =>
  [...Array(count)].map(hypervisor);
