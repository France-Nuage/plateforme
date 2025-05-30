import { createSlice } from '@reduxjs/toolkit';

import { ServiceMode } from '@/types';

export type ApplicationState = {
  mode: ServiceMode;
};

const initialState = {
  mode: ServiceMode.Mock,
};

export const applicationSlice = createSlice({
  name: 'application',
  initialState,
  reducers: {
    setMode: (state) => {
      state.mode =
        state.mode === ServiceMode.Rpc ? ServiceMode.Mock : ServiceMode.Rpc;
    },
  },
});

export const { setMode } = applicationSlice.actions;

export default applicationSlice.reducer;
