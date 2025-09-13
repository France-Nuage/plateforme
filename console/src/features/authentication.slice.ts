import { PayloadAction, createSlice } from '@reduxjs/toolkit';
import { createAsyncThunk } from '@reduxjs/toolkit';
import { User as OIDCUser } from 'oidc-client-ts';

import { User } from '@/types';

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
    setOIDCUser: (state, action: PayloadAction<OIDCUser>) => {
      state.token = action.payload.id_token;

      state.user = {
        email: action.payload.profile.email!,
        name: action.payload.profile.name,
        picture: action.payload.profile.picture,
      };
    },
  },
});

export const { clearAuthenticationState, setOIDCUser } =
  authenticationSlice.actions;

export default authenticationSlice;
