import config from '@/config';

/**
 * Identity payload returned by the control plane's `GET /auth/me` endpoint under
 * the confidential-client (BFF) flow.
 *
 * `authenticated` discriminates the two shapes: when `false`, the visitor has no
 * valid session and the other fields are empty. The `isAdmin` flag is resolved
 * server-side from `users.is_admin` (the same authoritative source as the
 * Profile.GetCurrentUser RPC), never from a token claim.
 */
export type Me = {
  authenticated: boolean;
  sub: string;
  email: string;
  firstName: string;
  lastName: string;
  picture: string;
  isAdmin: boolean;
};

/**
 * Reason a `/auth/callback` was rejected by the control plane.
 *
 * On any failure the control plane 302-redirects the browser to
 * `CONSOLE_URL?auth_error=<reason>` (see `bff.rs` `redirect_auth_error`); the
 * console reads it back to explain why the login did not complete instead of
 * silently retrying the flow and looping. These are the exact values emitted
 * server-side (`metrics::CallbackReject::as_str`).
 */
export type AuthErrorReason =
  | 'exchange'
  | 'no_id_token'
  | 'nonce'
  | 'session'
  | 'state'
  | 'validation';

/**
 * Redirects the browser to the server-side login endpoint, which starts the
 * OIDC authorization-code flow (state + nonce) against the identity provider.
 */
export function loginRedirect(): void {
  window.location.assign(`${config.controlplane}/auth/login`);
}

/**
 * Redirects the browser to the server-side logout endpoint, which clears the
 * session cookie and forwards to the identity provider's end-session endpoint.
 */
export function logoutRedirect(): void {
  window.location.assign(`${config.controlplane}/auth/logout`);
}

/**
 * Raised when `/auth/me` could not be completed by the control plane — either a
 * server error (HTTP 5xx) or a transport-level failure (control plane down,
 * DNS, connection refused, CORS) that rejects the fetch itself.
 *
 * This is distinct from an unauthenticated visitor: `/auth/me` answers `200`
 * with `{ authenticated: false }` when there is no session, so neither a 5xx nor
 * an unreachable control plane means "logged out". Surfacing it as its own error
 * lets the caller show a retry state instead of treating the visitor as logged
 * out and bouncing to `/login` — which would send the user to a dead
 * `/auth/login` (unreachable control plane) or loop through the identity
 * provider back to the same failing `/auth/me` (5xx).
 */
export class BffAuthServerError extends Error {
  constructor(detail: string) {
    super(`GET /auth/me could not be completed: ${detail}`);
    this.name = 'BffAuthServerError';
  }
}

/**
 * Reads the current session identity from the control plane.
 *
 * The request carries the httpOnly session cookie (`credentials: 'include'`);
 * the token itself is never exposed to JavaScript.
 *
 * A `5xx` — or a transport failure that rejects the fetch (control plane down,
 * DNS, connection refused, CORS) — is thrown as a {@link BffAuthServerError}
 * rather than parsed or allowed to fail closed: neither is the "logged out" case
 * (that is a `200` with `authenticated: false`), and treating them as logged out
 * would bounce the user to a dead `/auth/login`. Parsing a text/plain error body
 * as JSON would also mask a 5xx behind a generic parse failure.
 */
export function fetchMe(): Promise<Me> {
  return fetch(`${config.controlplane}/auth/me`, {
    credentials: 'include',
  })
    .then((response) => {
      if (response.status >= 500) {
        throw new BffAuthServerError(`server error ${response.status}`);
      }
      return response.json() as Promise<Me>;
    })
    .catch((error: unknown) => {
      // Re-throw the 5xx we just raised as-is; wrap anything else (a transport
      // rejection, or a malformed body from a broken control plane) so the
      // caller shows the retry card instead of falling through to a logout.
      if (error instanceof BffAuthServerError) {
        throw error;
      }
      throw new BffAuthServerError('control plane unreachable');
    });
}

/**
 * The three distinguishable results of a session refresh.
 *
 * - `'refreshed'`: the control plane rotated the cookie (fresh session).
 * - `'rejected'`: fail-closed dead session — the refresh cookie is definitively
 *   dead (a resolved non-ok response, e.g. HTTP 401 on `/auth/refresh`).
 * - `'unreachable'`: the control plane could not be reached (down / DNS /
 *   connection refused / CORS — a transport-level rejection). This is NOT a dead
 *   session: collapsing it into `'rejected'` would bounce the user to a dead
 *   `/auth/login` on a transient control-plane-unreachable race.
 */
export type RefreshOutcome = 'refreshed' | 'rejected' | 'unreachable';

/**
 * A single in-flight refresh, shared by every concurrent caller.
 *
 * When a burst of gRPC calls fail with `UNAUTHENTICATED` at once, they must not
 * each fire their own `/auth/refresh`: they all await this one promise, which
 * is cleared once it settles so a later expiry can refresh again.
 */
let refreshInFlight: Promise<RefreshOutcome> | undefined;

/**
 * Renews the session server-side via the httpOnly refresh cookie.
 *
 * Resolves to a three-state {@link RefreshOutcome}: `'refreshed'` when the
 * control plane rotated the cookie, `'rejected'` when a resolved response says
 * the refresh cookie is definitively dead (fail closed), and `'unreachable'`
 * when the fetch itself rejects (control plane down / DNS / connection refused /
 * CORS) — which is NOT a dead session. It NEVER throws and NEVER retries.
 *
 * This is a plain `fetch`, not a gRPC call, so a 401 here does NOT re-enter the
 * gRPC auth interceptor — there is no recursion.
 */
export function refreshSession(): Promise<RefreshOutcome> {
  if (refreshInFlight) {
    return refreshInFlight;
  }

  refreshInFlight = fetch(`${config.controlplane}/auth/refresh`, {
    credentials: 'include',
  })
    .then(
      (response): RefreshOutcome => (response.ok ? 'refreshed' : 'rejected'),
    )
    .catch((): RefreshOutcome => 'unreachable')
    .finally(() => {
      refreshInFlight = undefined;
    });

  return refreshInFlight;
}
