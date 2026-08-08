import { describe, expect, it } from 'vitest';

import type { Me } from '@/services/bff-auth';

import authenticationSlice, {
  clearAuthenticationState,
  fetchSession,
  logout,
} from './authentication.slice';

const reducer = authenticationSlice.reducer;

/**
 * Builds a `/auth/me` payload. Defaults to an unauthenticated visitor; pass the
 * fields relevant to the case under test.
 */
function me(overrides: Partial<Me>): Me {
  return {
    authenticated: false,
    email: '',
    firstName: '',
    isAdmin: false,
    lastName: '',
    picture: '',
    sub: '',
    ...overrides,
  };
}

const request = 'req';

describe('fetchSession bootstrap', () => {
  it('is unauthenticated and non-admin by default', () => {
    const state = reducer(undefined, { type: '@@INIT' });
    expect(state.authenticated).toBe(false);
    expect(state.isAdmin).toBe(false);
  });

  it('authenticates an admin session', () => {
    const state = reducer(
      undefined,
      fetchSession.fulfilled(
        me({ authenticated: true, isAdmin: true }),
        request,
        undefined,
      ),
    );
    expect(state.authenticated).toBe(true);
    expect(state.isAdmin).toBe(true);
  });

  it('authenticates a non-admin session', () => {
    const state = reducer(
      undefined,
      fetchSession.fulfilled(
        me({ authenticated: true, isAdmin: false }),
        request,
        undefined,
      ),
    );
    expect(state.authenticated).toBe(true);
    expect(state.isAdmin).toBe(false);
  });

  it('trusts the discriminant: no session means non-admin, even if the flag says otherwise', () => {
    const state = reducer(
      undefined,
      fetchSession.fulfilled(
        me({ authenticated: false, isAdmin: true }),
        request,
        undefined,
      ),
    );
    expect(state.authenticated).toBe(false);
    expect(state.isAdmin).toBe(false);
  });

  it('fails closed when /auth/me cannot be read', () => {
    const authenticated = reducer(
      undefined,
      fetchSession.fulfilled(
        me({ authenticated: true, isAdmin: true }),
        request,
        undefined,
      ),
    );
    const state = reducer(
      authenticated,
      fetchSession.rejected(new Error('network'), request, undefined),
    );
    expect(state.authenticated).toBe(false);
    expect(state.isAdmin).toBe(false);
  });
});

describe('logout and clear', () => {
  const authenticatedAdmin = reducer(
    undefined,
    fetchSession.fulfilled(
      me({ authenticated: true, isAdmin: true }),
      request,
      undefined,
    ),
  );

  it('clears the session on logout', () => {
    const state = reducer(
      authenticatedAdmin,
      logout.fulfilled(undefined, request, undefined),
    );
    expect(state.authenticated).toBe(false);
    expect(state.isAdmin).toBe(false);
  });

  it('clears the session on clearAuthenticationState', () => {
    const state = reducer(authenticatedAdmin, clearAuthenticationState());
    expect(state.authenticated).toBe(false);
    expect(state.isAdmin).toBe(false);
  });
});
