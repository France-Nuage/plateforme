import {
  MethodInfo,
  NextUnaryFn,
  RpcError,
  RpcMetadata,
  RpcOptions,
  UnaryCall,
} from '@protobuf-ts/runtime-rpc';
import { describe, expect, it } from 'vitest';

import { clearAuthenticationState } from '@/features';
import { AppStore } from '@/store';

import { createUnaryAuthInterceptor } from './rpc';

/**
 * A store that records every dispatched action, so the tests can assert whether
 * the session was cleared (fail-closed) without pulling in the real store.
 */
function createRecordingStore() {
  const dispatched: { type: string }[] = [];
  const store = {
    dispatch: (action: { type: string }) => {
      dispatched.push(action);
      return action;
    },
    state: {},
  } as unknown as AppStore;
  return { dispatched, store };
}

/**
 * Builds a minimal unary call whose only meaningful promise is `response`; the
 * others resolve so they never interfere with the assertions.
 */
function makeCall(response: Promise<object>): UnaryCall {
  const meta: RpcMetadata = {};
  return new UnaryCall(
    {} as MethodInfo,
    meta,
    {},
    Promise.resolve(meta),
    response,
    Promise.resolve({ code: 'OK', detail: '' }),
    Promise.resolve(meta),
  );
}

/**
 * A `next` that yields one response per invocation (repeating the last one), and
 * exposes how many times it was invoked — i.e. how many times the call ran.
 */
function recordingNext(factories: Array<() => Promise<object>>) {
  let index = 0;
  const next: NextUnaryFn = () => {
    const factory = factories[Math.min(index, factories.length - 1)];
    index += 1;
    return makeCall(factory());
  };
  return { count: () => index, next };
}

function unauthenticated(): RpcError {
  return new RpcError('unauthenticated', 'UNAUTHENTICATED');
}

function intercept(
  store: AppStore,
  refresh: () => Promise<boolean>,
  next: NextUnaryFn,
) {
  return createUnaryAuthInterceptor(store, refresh).interceptUnary(
    next,
    {} as MethodInfo,
    {},
    {} as RpcOptions,
  );
}

describe('createUnaryAuthInterceptor — bounded 401 recovery', () => {
  it('passes a successful call straight through (no refresh)', async () => {
    const { dispatched, store } = createRecordingStore();
    let refreshCalls = 0;
    const refresh = () => {
      refreshCalls += 1;
      return Promise.resolve(true);
    };
    const payload = { value: 'ok' };
    const { count, next } = recordingNext([() => Promise.resolve(payload)]);

    const call = intercept(store, refresh, next);

    await expect(call.response).resolves.toBe(payload);
    expect(refreshCalls).toBe(0);
    expect(count()).toBe(1);
    expect(dispatched).toHaveLength(0);
  });

  it('(a) refreshes once then replays the call once on a 401 → succeeds', async () => {
    const { dispatched, store } = createRecordingStore();
    let refreshCalls = 0;
    const refresh = () => {
      refreshCalls += 1;
      return Promise.resolve(true);
    };
    const payload = { value: 'ok' };
    const { count, next } = recordingNext([
      () => Promise.reject(unauthenticated()),
      () => Promise.resolve(payload),
    ]);

    const call = intercept(store, refresh, next);

    await expect(call.response).resolves.toBe(payload);
    expect(refreshCalls).toBe(1);
    expect(count()).toBe(2); // initial + exactly one replay
    expect(dispatched).toHaveLength(0); // no logout on success
  });

  it('(b) fails closed (clears auth, no replay, no loop) when the refresh fails', async () => {
    const { dispatched, store } = createRecordingStore();
    let refreshCalls = 0;
    const refresh = () => {
      refreshCalls += 1;
      return Promise.resolve(false);
    };
    const { count, next } = recordingNext([
      () => Promise.reject(unauthenticated()),
    ]);

    const call = intercept(store, refresh, next);

    await expect(call.response).rejects.toBeInstanceOf(RpcError);
    expect(refreshCalls).toBe(1);
    expect(count()).toBe(1); // never replayed — bounded
    expect(dispatched.map((action) => action.type)).toContain(
      clearAuthenticationState.type,
    );
  });

  it('fails closed when the replay still returns 401 (exactly one refresh, one replay)', async () => {
    const { dispatched, store } = createRecordingStore();
    let refreshCalls = 0;
    const refresh = () => {
      refreshCalls += 1;
      return Promise.resolve(true);
    };
    const { count, next } = recordingNext([
      () => Promise.reject(unauthenticated()),
      () => Promise.reject(unauthenticated()),
    ]);

    const call = intercept(store, refresh, next);

    await expect(call.response).rejects.toBeInstanceOf(RpcError);
    expect(refreshCalls).toBe(1); // no second refresh
    expect(count()).toBe(2); // initial + one replay, then stop
    expect(dispatched.map((action) => action.type)).toContain(
      clearAuthenticationState.type,
    );
  });

  it('propagates a non-auth error without refreshing or clearing', async () => {
    const { dispatched, store } = createRecordingStore();
    let refreshCalls = 0;
    const refresh = () => {
      refreshCalls += 1;
      return Promise.resolve(true);
    };
    const failure = new RpcError('boom', 'INTERNAL');
    const { count, next } = recordingNext([() => Promise.reject(failure)]);

    const call = intercept(store, refresh, next);

    await expect(call.response).rejects.toBe(failure);
    expect(refreshCalls).toBe(0);
    expect(count()).toBe(1);
    expect(dispatched).toHaveLength(0);
  });

  it('(c) concurrent 401s share the injected refresh path (one replay each, bounded)', async () => {
    const { store } = createRecordingStore();
    let refreshCalls = 0;
    const refresh = () => {
      refreshCalls += 1;
      return Promise.resolve(true);
    };
    const payload = { value: 'ok' };
    const first = recordingNext([
      () => Promise.reject(unauthenticated()),
      () => Promise.resolve(payload),
    ]);
    const second = recordingNext([
      () => Promise.reject(unauthenticated()),
      () => Promise.resolve(payload),
    ]);

    const callA = intercept(store, refresh, first.next);
    const callB = intercept(store, refresh, second.next);

    await Promise.all([
      expect(callA.response).resolves.toBe(payload),
      expect(callB.response).resolves.toBe(payload),
    ]);
    // Each call refreshes at most once and replays at most once. The de-duplication
    // of concurrent refreshes itself lives in refreshSession (single-flight), which
    // is covered in bff-auth.spec.ts.
    expect(refreshCalls).toBe(2);
    expect(first.count()).toBe(2);
    expect(second.count()).toBe(2);
  });
});
