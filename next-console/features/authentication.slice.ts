import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { User as OIDCUser } from "oidc-client-ts";
import { User } from "@/types";

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

/**
 * The authentication slice.
 */
export const authenticationSlice = createSlice({
  name: "authentication",
  initialState,
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
      action: PayloadAction<Pick<OIDCUser, "id_token" | "profile">>,
    ) => {
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
