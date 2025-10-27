import { faker } from '@faker-js/faker';

import { User } from '../models';

export const user = (seed?: Partial<User>): User => ({
  email: faker.internet.email(),
  firstName: faker.person.firstName(),
  lastName: faker.person.lastName(),
  username: faker.internet.username(),
  ...seed,
});

export const users = (count: number, seed?: Partial<User>): User[] =>
  [...Array(count)].map(() => user(seed));
