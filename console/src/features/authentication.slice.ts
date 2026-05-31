import { User } from '@france-nuage/sdk';
import { PayloadAction, createSlice } from '@reduxjs/toolkit';
import { createAsyncThunk } from '@reduxjs/toolkit';
import { User as OIDCUser } from 'oidc-client-ts';

/**
 * Represents the authentication state.
 */
type AuthenticationState = {
  token?: string;
  user?: User;
  /**
   * Whether the authenticated user has the admin role.
   *
   * NOTE: This is a UX-only hint derived from the 'admin' realm role in the
   * Keycloak id_token (`realm_access.roles`). The server-side `users.is_admin`
   * flag is the authoritative gate; this flag only hides/shows admin UI. For
   * this to work, Keycloak must be configured to map the 'admin' realm role
   * into the id_token, kept in sync with `users.is_admin`.
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

// todo: OIDCUser is a class, not an object, which does not serialize, so we parse it to smth better for redux
export function parseOidcUser(user: OIDCUser): {
  token: string;
  user: User;
  isAdmin: boolean;
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

  // Derive admin status from the Keycloak realm_access.roles claim.
  // The 'admin' role must be mapped into the id_token by Keycloak and kept
  // in sync with the server-side users.is_admin flag.
  const realmAccess = user.profile['realm_access'];
  const isAdmin =
    typeof realmAccess === 'object' &&
    realmAccess !== null &&
    Array.isArray((realmAccess as { roles?: unknown }).roles) &&
    (realmAccess as { roles: string[] }).roles.includes('admin');

  return {
    isAdmin,
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
     * Stores the isAdmin flag derived from the Keycloak realm_access.roles
     * claim. See the AuthenticationState.isAdmin field for caveats.
     */
    setOIDCUser: (
      state,
      action: PayloadAction<{ token: string; user: User; isAdmin: boolean }>,
    ) => {
      state.token = action.payload.token;
      state.isAdmin = action.payload.isAdmin;

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
