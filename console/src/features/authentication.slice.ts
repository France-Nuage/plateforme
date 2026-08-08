import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { Me, fetchMe, logoutRedirect } from '@/services/bff-auth';

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
};

/**
 * The initial authentication state, matching an unauthenticated user.
 */
const initialState: AuthenticationState = {
  authenticated: false,
  isAdmin: false,
};

/**
 * Bootstraps (or refreshes) the authentication state from the control plane.
 *
 * Reads `/auth/me` over the httpOnly session cookie and returns the identity
 * payload. The reducers below turn it into `authenticated` + `isAdmin`; a
 * rejected request fails closed (unauthenticated, non-admin).
 */
export const fetchSession = createAsyncThunk<Me, void>(
  'authentication/fetchSession',
  () => fetchMe(),
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
      if (me.authenticated) {
        state.authenticated = true;
        state.isAdmin = me.isAdmin;
      } else {
        state.authenticated = false;
        state.isAdmin = false;
      }
    });
    builder.addCase(fetchSession.rejected, (state) => {
      state.authenticated = false;
      state.isAdmin = false;
    });

    builder.addCase(logout.fulfilled, (state) => {
      state.authenticated = false;
      state.isAdmin = false;
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
    },
  },
});

export const { clearAuthenticationState } = authenticationSlice.actions;

export default authenticationSlice;
