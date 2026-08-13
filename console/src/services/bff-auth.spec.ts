import { afterEach, describe, expect, it, vi } from 'vitest';

import config from '@/config';

import {
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
      Promise.resolve({ json: () => Promise.resolve(payload) }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(fetchMe()).resolves.toEqual(payload);
    expect(fetchMock).toHaveBeenCalledWith(`${config.controlplane}/auth/me`, {
      credentials: 'include',
    });
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
    await expect(Promise.all([a, b, c])).resolves.toEqual([true, true, true]);

    // Once settled, the single-flight latch is released, so a later expiry can
    // refresh again.
    const d = refreshSession();
    expect(fetchCalls).toBe(2);
    resolveFetch({ ok: true });
    await expect(d).resolves.toBe(true);
  });

  it('(d) resolves false when /auth/refresh returns 401 (single fetch, no recursion)', async () => {
    let calls = 0;
    const fetchMock = vi.fn(() => {
      calls += 1;
      return Promise.resolve({ ok: false });
    });
    vi.stubGlobal('fetch', fetchMock);

    // A plain fetch, never retried: a 401 here cannot re-enter the gRPC
    // interceptor, so there is no recursion — it is a definitive fail-closed.
    await expect(refreshSession()).resolves.toBe(false);
    expect(calls).toBe(1);
  });

  it('resolves false on a network error', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.reject(new Error('offline'))),
    );

    await expect(refreshSession()).resolves.toBe(false);
  });
});
