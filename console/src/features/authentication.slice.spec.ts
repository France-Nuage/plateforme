import { User } from 'oidc-client-ts';
import { describe, expect, it } from 'vitest';

import { parseOidcUser } from './authentication.slice';

/**
 * Builds a minimal valid OIDCUser for testing.
 * Only the fields required by parseOidcUser are populated.
 */
function buildUser(overrides: {
  email?: string;
  family_name?: string;
  given_name?: string;
  id_token?: string;
  realm_access?: unknown;
}): User {
  return new User({
    access_token: 'access',
    id_token: overrides.id_token,
    profile: {
      email: overrides.email,
      family_name: overrides.family_name,
      given_name: overrides.given_name,
      sub: 'user-sub',
      ...(overrides.realm_access !== undefined
        ? { realm_access: overrides.realm_access }
        : {}),
    },
    session_state: null,
    token_type: 'Bearer',
  });
}

describe('parseOidcUser', () => {
  const validBase = {
    email: 'user@example.com',
    family_name: 'Dupont',
    given_name: 'Alice',
    id_token: 'tok',
  };

  it('returns isAdmin false when realm_access is absent', () => {
    const result = parseOidcUser(buildUser(validBase));
    expect(result.isAdmin).toBe(false);
  });

  it('returns isAdmin false when realm_access.roles does not include admin', () => {
    const result = parseOidcUser(
      buildUser({
        ...validBase,
        realm_access: { roles: ['viewer', 'editor'] },
      }),
    );
    expect(result.isAdmin).toBe(false);
  });

  it('returns isAdmin true when realm_access.roles includes admin', () => {
    const result = parseOidcUser(
      buildUser({ ...validBase, realm_access: { roles: ['viewer', 'admin'] } }),
    );
    expect(result.isAdmin).toBe(true);
  });

  it('throws when id_token is absent', () => {
    expect(() =>
      parseOidcUser(buildUser({ ...validBase, id_token: undefined })),
    ).toThrow();
  });

  it('throws when email is absent', () => {
    expect(() =>
      parseOidcUser(buildUser({ ...validBase, email: undefined })),
    ).toThrow();
  });

  it('maps token, user fields, and isAdmin correctly on a valid user', () => {
    const result = parseOidcUser(
      buildUser({ ...validBase, realm_access: { roles: ['admin'] } }),
    );
    expect(result.token).toBe('tok');
    expect(result.user.email).toBe('user@example.com');
    expect(result.user.firstName).toBe('Alice');
    expect(result.user.lastName).toBe('Dupont');
    expect(result.isAdmin).toBe(true);
  });
});
