import { CurrentUser, User } from '@france-nuage/sdk';
import { PayloadAction, createSlice } from '@reduxjs/toolkit';
import { createAsyncThunk } from '@reduxjs/toolkit';
import { User as OIDCUser } from 'oidc-client-ts';

import { ExtraArgument } from '@/store';

/**
 * Represents the authentication state.
 */
type AuthenticationState = {
  token?: string;
  user?: User;
  /**
   * Whether the authenticated user holds platform-admin privileges.
   *
   * Authoritative value fetched from the control plane via `fetchCurrentUser`
   * (which reads `users.is_admin`). Keycloak only authenticates the user; it is
   * never inspected for roles. Defaults to false until confirmed by the server.
   */
  isAdmin: boolean;
};

/**
 * The initial authentication state, matching an unauthenticated user.
 */
const initialState: AuthenticationState = {
  isAdmin: false,
};

export const logout = createAsyncThunk<void, void>(
  'authentication/logout',
  async () => {},
);

/**
 * Fetches the authenticated caller's platform-admin status from the control
 * plane. This is the single source of truth for `isAdmin`: it reflects
 * `users.is_admin` in the database, not an OIDC token claim. Dispatch it right
 * after `setOIDCUser`, once the bearer token is in the store.
 */
export const fetchCurrentUser = createAsyncThunk<
  CurrentUser,
  void,
  { extra: ExtraArgument }
>('authentication/fetchCurrentUser', (_, { extra }) =>
  extra.services.profile.getCurrentUser(),
);

// todo: OIDCUser is a class, not an object, which does not serialize, so we parse it to smth better for redux
export function parseOidcUser(user: OIDCUser): {
  token: string;
  user: User;
} {
  if (
    !user.id_token ||
    !user.profile ||
    !user.profile.email ||
    !user.profile.given_name ||
    !user.profile.family_name
  ) {
    throw new Error('Error: user format is not valid.');
  }

  return {
    token: user.id_token,
    user: {
      email: user.profile.email,
      firstName: user.profile.given_name,
      lastName: user.profile.family_name,
      picture: user.profile.picture,
    },
  };
}

/**
 * The authentication slice.
 */
export const authenticationSlice = createSlice({
  extraReducers: (builder) => {
    builder.addCase(logout.fulfilled, (state) => {
      state.token = undefined;
      state.user = undefined;
      state.isAdmin = false;
    });

    // Authoritative admin status, resolved server-side. On failure we fail
    // closed (no admin UI) rather than trusting a stale value.
    builder.addCase(fetchCurrentUser.fulfilled, (state, action) => {
      state.isAdmin = action.payload.isAdmin;
    });
    builder.addCase(fetchCurrentUser.rejected, (state) => {
      state.isAdmin = false;
    });
  },
  initialState,
  name: 'authentication',
  reducers: {
    /**
     * Clears the authentication state, reverting back to an unauthenticated state.
     */
    clearAuthenticationState: (state) => {
      state.token = undefined;
      state.user = undefined;
      state.isAdmin = false;
    },
    /**
     * Set the authentication state to represent a logged in user.
     *
     * Stores the OIDC identity and bearer token. The admin status is resolved
     * separately via `fetchCurrentUser`, which must be dispatched afterwards.
     */
    setOIDCUser: (
      state,
      action: PayloadAction<{ token: string; user: User }>,
    ) => {
      state.token = action.payload.token;

      state.user = {
        email: action.payload.user.email,
        firstName: action.payload.user.firstName,
        lastName: action.payload.user.lastName,
        picture: action.payload.user.picture,
      };
    },
  },
});

export const { clearAuthenticationState, setOIDCUser } =
  authenticationSlice.actions;

export default authenticationSlice;
