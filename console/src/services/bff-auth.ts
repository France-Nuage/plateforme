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
 * Raised when `/auth/me` fails with a server error (HTTP 5xx).
 *
 * This is distinct from an unauthenticated visitor: `/auth/me` answers `200`
 * with `{ authenticated: false }` when there is no session, so a 5xx means the
 * control plane itself is broken. Surfacing it as its own error lets the caller
 * show an error state instead of treating the visitor as logged out and
 * bouncing to `/login` — which would loop through the identity provider back to
 * the same failing `/auth/me`.
 */
export class BffAuthServerError extends Error {
  constructor(status: number) {
    super(`GET /auth/me failed with server error ${status}`);
    this.name = 'BffAuthServerError';
  }
}

/**
 * Reads the current session identity from the control plane.
 *
 * The request carries the httpOnly session cookie (`credentials: 'include'`);
 * the token itself is never exposed to JavaScript.
 *
 * A `5xx` is thrown as a {@link BffAuthServerError} rather than parsed: a
 * server error is NOT the "logged out" case (that is a `200` with
 * `authenticated: false`), and parsing a text/plain error body as JSON would
 * mask it behind a generic parse failure.
 */
export function fetchMe(): Promise<Me> {
  return fetch(`${config.controlplane}/auth/me`, {
    credentials: 'include',
  }).then((response) => {
    if (response.status >= 500) {
      throw new BffAuthServerError(response.status);
    }
    return response.json() as Promise<Me>;
  });
}

/**
 * A single in-flight refresh, shared by every concurrent caller.
 *
 * When a burst of gRPC calls fail with `UNAUTHENTICATED` at once, they must not
 * each fire their own `/auth/refresh`: they all await this one promise, which
 * is cleared once it settles so a later expiry can refresh again.
 */
let refreshInFlight: Promise<boolean> | undefined;

/**
 * Renews the session server-side via the httpOnly refresh cookie.
 *
 * Resolves to `true` when the control plane rotated the cookie (the browser now
 * holds a fresh session), `false` on any failure — an expired refresh cookie
 * (HTTP 401 on `/auth/refresh` itself) or a network error. It NEVER throws and
 * NEVER retries, so the caller can treat `false` as a definitive "session is
 * dead, fail closed".
 *
 * This is a plain `fetch`, not a gRPC call, so a 401 here does NOT re-enter the
 * gRPC auth interceptor — there is no recursion.
 */
export function refreshSession(): Promise<boolean> {
  if (refreshInFlight) {
    return refreshInFlight;
  }

  refreshInFlight = fetch(`${config.controlplane}/auth/refresh`, {
    credentials: 'include',
  })
    .then((response) => response.ok)
    .catch(() => false)
    .finally(() => {
      refreshInFlight = undefined;
    });

  return refreshInFlight;
}
