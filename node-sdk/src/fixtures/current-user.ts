import { faker } from '@faker-js/faker';

import { CurrentUser } from '../models';

/**
 * A fixture current user. Defaults to a platform admin so the Mock service mode
 * exercises the admin-gated UI during local development.
 */
export const currentUser = (
  overrides: Partial<CurrentUser> = {},
): CurrentUser => ({
  id: faker.string.uuid(),
  email: faker.internet.email(),
  isAdmin: true,
  ...overrides,
});
