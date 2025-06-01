import { faker } from '@faker-js/faker';
import { Organization } from "@/types";

export const organization = (): Organization => ({
  id: faker.string.uuid(),
  name: faker.commerce.productName(),
});

export const organizations = (count: number): Organization[] => [...Array(count)].map(organization);
