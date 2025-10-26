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
};

/**
 * The initial authentication state, matching an unauthenticated user.
 */
const initialState: AuthenticationState = {};

export const logout = createAsyncThunk<void, void>(
  'authentication/logout',
  async () => {},
);

// todo: OIDCUser is a class, not an object, which does not serialize, so we parse it to smth better for redux
export function parseOidcUser(user: OIDCUser): { token: string; user: User } {
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
    },
    /**
     * Set the authenthentication state to represent a logged in user.
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
