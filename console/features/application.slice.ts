import { ServiceMode } from "@/types";
import { createSlice, PayloadAction } from "@reduxjs/toolkit";

export type ApplicationState = {
  mode: ServiceMode;
};

const initialState = {
  mode: ServiceMode.Mock,
};

export const applicationSlice = createSlice({
  name: "application",
  initialState,
  reducers: {
    setMode: (state, action: PayloadAction<void>) => {
      state.mode =
        state.mode === ServiceMode.Rpc ? ServiceMode.Mock : ServiceMode.Rpc;
    },
  },
});

export const { setMode } = applicationSlice.actions;

export default applicationSlice.reducer;
