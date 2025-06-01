import { faker } from '@faker-js/faker';
import { Project } from "@/types";

export const project = (): Project => ({
  id: faker.string.uuid(),
  name: faker.commerce.productName(),
  organizationId: faker.string.uuid(),
});

export const projects = (count: number): Project[] => [...Array(count)].map(project);
