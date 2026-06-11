import { User } from 'oidc-client-ts';
import { describe, expect, it } from 'vitest';

import authenticationSlice, {
  clearAuthenticationState,
  fetchCurrentUser,
  logout,
  parseOidcUser,
  setOIDCUser,
} from './authentication.slice';

const reducer = authenticationSlice.reducer;

/**
 * Builds a minimal valid OIDCUser for testing.
 * Only the fields required by parseOidcUser are populated.
 */
function buildUser(overrides: {
  email?: string;
  family_name?: string;
  given_name?: string;
  id_token?: string;
}): User {
  return new User({
    access_token: 'access',
    id_token: overrides.id_token,
    profile: {
      aud: 'audience',
      email: overrides.email,
      exp: 0,
      family_name: overrides.family_name,
      given_name: overrides.given_name,
      iat: 0,
      iss: 'issuer',
      sub: 'user-sub',
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

  it('maps token and user fields on a valid user', () => {
    const result = parseOidcUser(buildUser(validBase));
    expect(result.token).toBe('tok');
    expect(result.user.email).toBe('user@example.com');
    expect(result.user.firstName).toBe('Alice');
    expect(result.user.lastName).toBe('Dupont');
  });

  it('does not derive admin status from the token', () => {
    const result = parseOidcUser(buildUser(validBase));
    expect('isAdmin' in result).toBe(false);
  });
});

describe('authentication reducer admin status', () => {
  it('is not admin by default', () => {
    const state = reducer(undefined, { type: '@@INIT' });
    expect(state.isAdmin).toBe(false);
  });

  it('becomes admin when the control plane confirms it', () => {
    const state = reducer(
      undefined,
      fetchCurrentUser.fulfilled(
        { email: 'a@b.c', id: 'id', isAdmin: true },
        'req',
        undefined,
      ),
    );
    expect(state.isAdmin).toBe(true);
  });

  it('stays non-admin when the control plane denies it', () => {
    const state = reducer(
      undefined,
      fetchCurrentUser.fulfilled(
        { email: 'a@b.c', id: 'id', isAdmin: false },
        'req',
        undefined,
      ),
    );
    expect(state.isAdmin).toBe(false);
  });

  it('fails closed (drops admin) when the lookup fails', () => {
    const admin = reducer(
      undefined,
      fetchCurrentUser.fulfilled(
        { email: 'a@b.c', id: 'id', isAdmin: true },
        'req',
        undefined,
      ),
    );
    const state = reducer(
      admin,
      fetchCurrentUser.rejected(new Error('network'), 'req', undefined),
    );
    expect(state.isAdmin).toBe(false);
  });

  it('does not change admin status when setting the OIDC user', () => {
    const admin = reducer(
      undefined,
      fetchCurrentUser.fulfilled(
        { email: 'a@b.c', id: 'id', isAdmin: true },
        'req',
        undefined,
      ),
    );
    const state = reducer(
      admin,
      setOIDCUser({
        token: 'tok',
        user: { email: 'a@b.c', firstName: 'Alice', lastName: 'Dupont' },
      }),
    );
    expect(state.token).toBe('tok');
    expect(state.isAdmin).toBe(true);
  });

  it('clears admin status on logout', () => {
    const admin = reducer(
      undefined,
      fetchCurrentUser.fulfilled(
        { email: 'a@b.c', id: 'id', isAdmin: true },
        'req',
        undefined,
      ),
    );
    const state = reducer(admin, logout.fulfilled(undefined, 'req', undefined));
    expect(state.isAdmin).toBe(false);
    expect(state.token).toBeUndefined();
  });

  it('clears admin status when clearing the auth state', () => {
    const admin = reducer(
      undefined,
      fetchCurrentUser.fulfilled(
        { email: 'a@b.c', id: 'id', isAdmin: true },
        'req',
        undefined,
      ),
    );
    const state = reducer(admin, clearAuthenticationState());
    expect(state.isAdmin).toBe(false);
  });
});
