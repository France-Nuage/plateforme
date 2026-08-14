import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import {
  BffAuthServerError,
  Me,
  fetchMe,
  logoutRedirect,
  refreshSession,
} from '@/services/bff-auth';

/**
 * Represents the authentication state.
 *
 * The whole state is derived from the control plane's `/auth/me` endpoint
 * (see {@link fetchSession}): the browser never inspects a token. `isAdmin`
 * reflects `users.is_admin` in the database, resolved server-side.
 */
type AuthenticationState = {
  /**
   * Whether the visitor has a valid server-side session.
   */
  authenticated: boolean;
  /**
   * Whether the authenticated user holds platform-admin privileges.
   *
   * Authoritative value sourced from `/auth/me` (which reads `users.is_admin`).
   * The identity provider only authenticates the user; it is never inspected
   * for roles. Defaults to false until confirmed by the server.
   */
  isAdmin: boolean;
  /**
   * Whether the last `/auth/me` call failed with a server error (HTTP 5xx).
   *
   * This is distinct from being unauthenticated: a 5xx means the control plane
   * is broken, not that the visitor is logged out. It lets the app show an
   * error state instead of bouncing to `/login`, which on a persistent 5xx
   * would loop through the identity provider back to the same failing endpoint.
   */
  sessionError: boolean;
};

/**
 * The initial authentication state, matching an unauthenticated user.
 */
const initialState: AuthenticationState = {
  authenticated: false,
  isAdmin: false,
  sessionError: false,
};

/**
 * Bootstraps (or refreshes) the authentication state from the control plane.
 *
 * Reads `/auth/me` over the httpOnly session cookie and returns the identity
 * payload. The reducers below turn it into `authenticated` + `isAdmin`.
 *
 * When `/auth/me` reports no session, the short-lived access token may simply
 * have lapsed while the longer-lived (12h) session cookie is still refreshable.
 * A single silent {@link refreshSession} is attempted before giving up: on
 * success `/auth/me` is read once more and that result is used; otherwise the
 * unauthenticated payload stands. This is bounded to one attempt (no recursion,
 * no loop) so a genuinely dead session still resolves to unauthenticated and
 * lets the page guard redirect to `/login`, instead of every post-expiry page
 * load paying a full identity-provider round-trip.
 *
 * A control-plane 5xx ({@link BffAuthServerError}) is rejected with the
 * `'server-error'` value so the reducer can flag `sessionError` and the app can
 * show an error state — it is thrown before the refresh branch, so a broken
 * control plane never triggers a refresh. Every other failure (network,
 * unexpected) fails closed (unauthenticated, non-admin).
 */
export const fetchSession = createAsyncThunk<
  Me,
  void,
  { rejectValue: 'server-error' }
>('authentication/fetchSession', (_, { rejectWithValue }) =>
  fetchMe()
    .then((me) => {
      if (me.authenticated) {
        return me;
      }
      return refreshSession().then((refreshed) => {
        if (refreshed) {
          return fetchMe();
        }
        return me;
      });
    })
    .catch((error) => {
      if (error instanceof BffAuthServerError) {
        return rejectWithValue('server-error');
      }
      throw error;
    }),
);

/**
 * Logs the user out.
 *
 * Redirects the browser to the control plane's `/auth/logout`, which clears the
 * session cookie server-side and forwards to the identity provider's end-session
 * endpoint. The local state is cleared as well (moot once the navigation
 * happens, but keeps the store consistent if it does not).
 */
export const logout = createAsyncThunk<void, void>(
  'authentication/logout',
  () => {
    logoutRedirect();
    return Promise.resolve();
  },
);

/**
 * The authentication slice.
 */
export const authenticationSlice = createSlice({
  extraReducers: (builder) => {
    // Session identity resolved server-side. On success we trust the
    // `authenticated` discriminant; on failure we fail closed.
    builder.addCase(fetchSession.fulfilled, (state, action) => {
      const me = action.payload;
      state.sessionError = false;
      if (me.authenticated) {
        state.authenticated = true;
        state.isAdmin = me.isAdmin;
      } else {
        state.authenticated = false;
        state.isAdmin = false;
      }
    });
    builder.addCase(fetchSession.rejected, (state, action) => {
      state.authenticated = false;
      state.isAdmin = false;
      // A 5xx (`server-error`) is a broken control plane, not a logged-out
      // visitor: flag it so the app shows an error state instead of bouncing to
      // `/login`. Any other rejection (network, unexpected) fails closed.
      state.sessionError = action.payload === 'server-error';
    });

    builder.addCase(logout.fulfilled, (state) => {
      state.authenticated = false;
      state.isAdmin = false;
      state.sessionError = false;
    });
  },
  initialState,
  name: 'authentication',
  reducers: {
    /**
     * Clears the authentication state, reverting back to an unauthenticated
     * state. Dispatched when the session cannot be recovered (see the gRPC auth
     * interceptor), which lets the page guard bounce the user to `/login`.
     */
    clearAuthenticationState: (state) => {
      state.authenticated = false;
      state.isAdmin = false;
      state.sessionError = false;
    },
  },
});

export const { clearAuthenticationState } = authenticationSlice.actions;

export default authenticationSlice;
