import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { User as OIDCUser } from "oidc-client-ts";
import { User } from "@/types";

type AuthenticationState = {
  token?: string;
  user?: User;
}


const initialState: AuthenticationState = {}

export const authenticationSlice = createSlice({
  name: "authentication",
  initialState,
  reducers: {
    clearAuthenticationState: (state) => {
      state.token = undefined;
      state.user = undefined;
    },
    setOIDCUser: (state, action: PayloadAction<Pick<OIDCUser, 'id_token' | 'profile'>>) => {
      state.token = action.payload.id_token;
      state.user = {
        email: action.payload.profile.email!,
        name: action.payload.profile.name,
        picture: action.payload.profile.picture,
      }
    },
  },
});

export const { clearAuthenticationState, setOIDCUser } = authenticationSlice.actions;

export default authenticationSlice;
