import { configureStore } from '@reduxjs/toolkit';
import { afterEach, describe, expect, it, vi } from 'vitest';

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

describe('fetchSession server error vs unauthenticated', () => {
  it('is not in error by default', () => {
    const state = reducer(undefined, { type: '@@INIT' });
    expect(state.sessionError).toBe(false);
  });

  it('flags a server error (5xx) so the app does not bounce to /login', () => {
    const state = reducer(
      undefined,
      fetchSession.rejected(
        new Error('Rejected'),
        request,
        undefined,
        'server-error',
      ),
    );
    expect(state.sessionError).toBe(true);
    expect(state.authenticated).toBe(false);
    expect(state.isAdmin).toBe(false);
  });

  it('does not flag a server error on an ordinary (network) rejection', () => {
    const state = reducer(
      undefined,
      fetchSession.rejected(new Error('network'), request, undefined),
    );
    expect(state.sessionError).toBe(false);
    expect(state.authenticated).toBe(false);
  });

  it('clears a previous server error once /auth/me succeeds again', () => {
    const errored = reducer(
      undefined,
      fetchSession.rejected(
        new Error('Rejected'),
        request,
        undefined,
        'server-error',
      ),
    );
    expect(errored.sessionError).toBe(true);

    const recovered = reducer(
      errored,
      fetchSession.fulfilled(
        me({ authenticated: true, isAdmin: false }),
        request,
        undefined,
      ),
    );
    expect(recovered.sessionError).toBe(false);
    expect(recovered.authenticated).toBe(true);
  });
});

describe('fetchSession refreshes before giving up', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  function jsonResponse(body: Me): { json: () => Promise<Me>; status: number } {
    return { json: () => Promise.resolve(body), status: 200 };
  }

  /**
   * Splits `fetch` calls between `/auth/me` (successive bodies from the queue)
   * and `/auth/refresh` (fixed ok flag), and records how often each is hit.
   */
  function stubBff(meBodies: Me[], refreshOk: boolean) {
    const meBodyQueue = [...meBodies];
    const calls = { me: 0, refresh: 0 };
    const fetchMock = vi.fn((url: string) => {
      if (url.endsWith('/auth/refresh')) {
        calls.refresh += 1;
        return Promise.resolve({ ok: refreshOk });
      }
      calls.me += 1;
      const body = meBodyQueue.shift();
      if (!body) {
        throw new Error(`unexpected extra /auth/me call #${calls.me}`);
      }
      return Promise.resolve(jsonResponse(body));
    });
    vi.stubGlobal('fetch', fetchMock);
    return calls;
  }

  function store() {
    return configureStore({
      reducer: { [authenticationSlice.name]: authenticationSlice.reducer },
    });
  }

  it('refreshes once and re-fetches when the session is initially unauthenticated but refreshable', async () => {
    const calls = stubBff(
      [
        me({ authenticated: false }),
        me({ authenticated: true, isAdmin: true }),
      ],
      true,
    );
    const app = store();

    await app.dispatch(fetchSession());

    expect(app.getState().authentication.authenticated).toBe(true);
    expect(app.getState().authentication.isAdmin).toBe(true);
    expect(calls.refresh).toBe(1); // exactly one refresh attempt
    expect(calls.me).toBe(2); // initial read + one re-fetch after refresh
  });

  it('stays unauthenticated after a single failed refresh, without looping', async () => {
    const calls = stubBff([me({ authenticated: false })], false);
    const app = store();

    await app.dispatch(fetchSession());

    expect(app.getState().authentication.authenticated).toBe(false);
    expect(app.getState().authentication.isAdmin).toBe(false);
    expect(calls.refresh).toBe(1); // one attempt, then it gives up
    expect(calls.me).toBe(1); // no re-fetch when the refresh failed
  });

  it('flags sessionError (retry card, not logout) when the refresh is unreachable', async () => {
    // /auth/me says logged-out, then /auth/refresh transport-fails: the control
    // plane is unreachable, NOT a dead session. It must surface as sessionError
    // (retry card), not fall through to unauthenticated + a /login bounce.
    const calls = { me: 0, refresh: 0 };
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string) => {
        if (url.endsWith('/auth/refresh')) {
          calls.refresh += 1;
          return Promise.reject(new Error('offline'));
        }
        calls.me += 1;
        return Promise.resolve(jsonResponse(me({ authenticated: false })));
      }),
    );
    const app = store();

    await app.dispatch(fetchSession());

    expect(app.getState().authentication.sessionError).toBe(true);
    expect(app.getState().authentication.authenticated).toBe(false);
    expect(calls.refresh).toBe(1); // one attempt
    expect(calls.me).toBe(1); // no re-fetch — the control plane is unreachable
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
