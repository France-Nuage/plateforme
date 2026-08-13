import {
  MethodInfo,
  NextUnaryFn,
  RpcError,
  RpcOptions,
  UnaryCall,
} from '@protobuf-ts/runtime-rpc';
import { debounce } from 'lodash';

import { ERROR_DEBOUNCE_WAIT } from '@/constants';
import { clearAuthenticationState } from '@/features';
import { AppStore } from '@/store';
import { toaster } from '@/toaster';

/**
 * Renews the server-side session, resolving `true` when the session was
 * refreshed and `false` when it is definitively dead. Injected so the retry
 * behaviour can be exercised in isolation.
 */
export type RefreshFn = () => Promise<boolean>;

/**
 * A non-thenable box around a `UnaryCall`.
 *
 * `UnaryCall` is itself a `PromiseLike`, so resolving a promise *to* a call
 * would auto-unwrap it to a `FinishedUnaryCall`. Boxing it keeps the call intact
 * as it flows through the recovery promise.
 */
type CallHolder = { call: UnaryCall };

/**
 * Builds a gRPC-web unary interceptor that recovers from an expired session,
 * exactly once.
 *
 * Authentication itself is carried by the httpOnly session cookie (the
 * transport is configured with `credentials: 'include'`), so there is no header
 * to add here. What this interceptor adds is bounded recovery: when a call
 * fails with `UNAUTHENTICATED` it
 *   1. refreshes the session (single-flight — concurrent 401s share one refresh),
 *   2. replays the failed call exactly once if the refresh succeeded,
 *   3. otherwise, or if the replay still returns `UNAUTHENTICATED`, clears the
 *      auth state so the page guard redirects the user to `/login`.
 *
 * The recovery is strictly bounded: at most one refresh and one replay per call,
 * never a loop. `/auth/refresh` is a plain fetch performed outside this
 * interceptor, so a 401 on the refresh itself resolves to `false` here and
 * cannot recurse back into the interceptor.
 */
export function createUnaryAuthInterceptor(
  store: AppStore,
  refresh: RefreshFn,
) {
  return {
    interceptUnary(
      next: NextUnaryFn,
      method: MethodInfo,
      input: object,
      options: RpcOptions,
    ): UnaryCall {
      const initial = next(method, input, options);

      // Resolve to whichever call is authoritative: the initial one on success,
      // or a single replay after a successful refresh. Rejects (after running
      // the relevant side effect) when the call cannot be recovered.
      const authoritative: Promise<CallHolder> = initial.response.then(
        () => ({ call: initial }),
        (error) => recover(error, next, method, input, options, store, refresh),
      );

      const headers = authoritative.then((holder) => holder.call.headers);
      const response = authoritative.then((holder) => holder.call.response);
      const status = authoritative.then((holder) => holder.call.status);
      const trailers = authoritative.then((holder) => holder.call.trailers);

      // Mirror protobuf-ts's Deferred: prevent an "unhandled rejection" warning
      // on the promises a caller does not await, without swallowing the
      // rejection for a caller that does await them.
      headers.catch(() => {});
      status.catch(() => {});
      trailers.catch(() => {});

      return new UnaryCall(
        method,
        initial.requestHeaders,
        initial.request,
        headers,
        response,
        status,
        trailers,
      );
    },
  };
}

/**
 * Attempts to recover a failed unary call. Returns the authoritative call on
 * success, or a rejected promise (after clearing auth / notifying) on failure.
 */
function recover(
  error: unknown,
  next: NextUnaryFn,
  method: MethodInfo,
  input: object,
  options: RpcOptions,
  store: AppStore,
  refresh: RefreshFn,
): Promise<CallHolder> {
  // Not an auth failure: nothing to recover, surface it to the caller.
  if (!isUnauthenticated(error)) {
    reportTerminalError(error);
    return Promise.reject(error);
  }

  return refresh().then((refreshed) => {
    // Session is dead: fail closed. The page guard reacts to the cleared state
    // and redirects to `/login`.
    if (!refreshed) {
      store.dispatch(clearAuthenticationState());
      return Promise.reject(error);
    }

    // Session refreshed: replay the call exactly once.
    const replay = next(method, input, options);
    return replay.response.then(
      () => ({ call: replay }),
      (replayError) => {
        // Still unauthenticated after a fresh session: give up, fail closed.
        if (isUnauthenticated(replayError)) {
          store.dispatch(clearAuthenticationState());
        } else {
          reportTerminalError(replayError);
        }
        return Promise.reject(replayError);
      },
    );
  });
}

/**
 * Type guard for an `UNAUTHENTICATED` gRPC error.
 */
function isUnauthenticated(error: unknown): error is RpcError {
  return error instanceof RpcError && error.code === 'UNAUTHENTICATED';
}

/**
 * Notifies the user of an unrecoverable RPC error via a toast.
 */
function reportTerminalError(error: unknown): void {
  if (!(error instanceof RpcError)) {
    console.error('unhandled non-rpc error', error);
    return;
  }
  error.message = decodeURIComponent(error.message);
  const mapped = mapUnregisteredErrorToAlphaVersionContextualError(error);
  notify(mapped.code, mapped.message);
}

/**
 * Notify an error to the user.
 *
 * The notification is displayed as a toast in the application UI. It is also
 * debounced, meaning it is displayed only once every `ERROR_DEBOUNCE_WAIT`
 * milliseconds. This is particularly convenient when multiple unary calls
 * (a.k.a. rpcs) fail with the same error.
 *
 * The notify function is instantiated once in the module rather than on every
 * call to the error handler to enable debouncing on the app level instead of
 * the call level.
 */
const notify = debounce((title: string, description: string) => {
  toaster.create({ description, title });
}, ERROR_DEBOUNCE_WAIT);

/**
 * Maps unregistered user errors to a contextual error with support contact.
 *
 * Temporary workaround for alpha version - users must be manually created in database.
 * Will be removed once SpiceDB and proper user onboarding are implemented.
 *
 * @param error - RPC error to transform
 * @returns Modified error with contextual message or original error
 */
function mapUnregisteredErrorToAlphaVersionContextualError(error: RpcError) {
  const regex = /^user\s+(\S+@\S+\.\S+)\s+is\s+not\s+registered$/;
  const match = error.message.match(regex);
  if (match) {
    const email = match[1];
    error.code = 'ACCESS DENIED';
    error.message = `Email "${email}" is not registered. Contact support@france-nuage.fr for alpha access.`;
  }

  return error;
}
