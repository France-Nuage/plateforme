import { afterEach, describe, expect, it, vi } from 'vitest';

import config from '@/config';

import {
  BffAuthServerError,
  fetchMe,
  loginRedirect,
  logoutRedirect,
  refreshSession,
} from './bff-auth';

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('redirects', () => {
  it('login redirects the browser to /auth/login', () => {
    const assign = vi.fn();
    vi.stubGlobal('window', { location: { assign } });

    loginRedirect();

    expect(assign).toHaveBeenCalledWith(`${config.controlplane}/auth/login`);
  });

  it('logout redirects the browser to /auth/logout', () => {
    const assign = vi.fn();
    vi.stubGlobal('window', { location: { assign } });

    logoutRedirect();

    expect(assign).toHaveBeenCalledWith(`${config.controlplane}/auth/logout`);
  });
});

describe('fetchMe', () => {
  it('reads /auth/me with the session cookie', async () => {
    const payload = {
      authenticated: true,
      email: 'a@b.c',
      firstName: '',
      isAdmin: true,
      lastName: '',
      picture: '',
      sub: 's',
    };
    const fetchMock = vi.fn(() =>
      Promise.resolve({ json: () => Promise.resolve(payload), status: 200 }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(fetchMe()).resolves.toEqual(payload);
    expect(fetchMock).toHaveBeenCalledWith(`${config.controlplane}/auth/me`, {
      credentials: 'include',
    });
  });

  it('returns the unauthenticated payload on a 200 (a logged-out visitor)', async () => {
    const payload = {
      authenticated: false,
      email: '',
      firstName: '',
      isAdmin: false,
      lastName: '',
      picture: '',
      sub: '',
    };
    vi.stubGlobal(
      'fetch',
      vi.fn(() =>
        Promise.resolve({ json: () => Promise.resolve(payload), status: 200 }),
      ),
    );

    await expect(fetchMe()).resolves.toEqual(payload);
  });

  it('throws BffAuthServerError on a 5xx instead of treating it as logged out', async () => {
    // A 500 with a text/plain body: `.json()` would throw and be mistaken for
    // "unauthenticated" (→ redirect loop). It must surface as a server error,
    // and the body must never be parsed as JSON.
    const json = vi.fn(() => Promise.reject(new Error('not json')));
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve({ json, status: 500 })),
    );

    await expect(fetchMe()).rejects.toBeInstanceOf(BffAuthServerError);
    expect(json).not.toHaveBeenCalled();
  });

  it('throws BffAuthServerError on a transport failure instead of failing closed to logout', async () => {
    // The control plane is unreachable (down / DNS / connection refused / CORS):
    // the fetch itself rejects. This is NOT "logged out" — it must surface as a
    // server error (retry card), never bounce the user to a dead /auth/login.
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.reject(new TypeError('Failed to fetch'))),
    );

    await expect(fetchMe()).rejects.toBeInstanceOf(BffAuthServerError);
  });
});

describe('refreshSession', () => {
  it('(c) coalesces concurrent refreshes into a single request (single-flight)', async () => {
    let resolveFetch: (value: { ok: boolean }) => void = () => {};
    let fetchCalls = 0;
    const fetchMock = vi.fn(
      () =>
        new Promise((resolve) => {
          fetchCalls += 1;
          resolveFetch = resolve;
        }),
    );
    vi.stubGlobal('fetch', fetchMock);

    const a = refreshSession();
    const b = refreshSession();
    const c = refreshSession();
    expect(fetchCalls).toBe(1); // all three share one in-flight request

    resolveFetch({ ok: true });
    await expect(Promise.all([a, b, c])).resolves.toEqual([
      'refreshed',
      'refreshed',
      'refreshed',
    ]);

    // Once settled, the single-flight latch is released, so a later expiry can
    // refresh again.
    const d = refreshSession();
    expect(fetchCalls).toBe(2);
    resolveFetch({ ok: true });
    await expect(d).resolves.toBe('refreshed');
  });

  it("(d) resolves 'rejected' when /auth/refresh returns 401 (single fetch, no recursion)", async () => {
    let calls = 0;
    const fetchMock = vi.fn(() => {
      calls += 1;
      return Promise.resolve({ ok: false });
    });
    vi.stubGlobal('fetch', fetchMock);

    // A plain fetch, never retried: a 401 here cannot re-enter the gRPC
    // interceptor, so there is no recursion — it is a definitive fail-closed
    // dead session.
    await expect(refreshSession()).resolves.toBe('rejected');
    expect(calls).toBe(1);
  });

  it("resolves 'unreachable' on a transport failure (NOT a dead session)", async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.reject(new Error('offline'))),
    );

    // The control plane could not be reached — the caller must not treat this
    // as a dead session and bounce to a dead /auth/login.
    await expect(refreshSession()).resolves.toBe('unreachable');
  });
});
