import { createSlice } from '@reduxjs/toolkit';

import { ServiceMode } from '@/types';

export type ApplicationState = {
  mode: ServiceMode;
};

const initialState = {
  mode: window.location.pathname.startsWith('/plasmic-host')
    ? ServiceMode.Mock
    : import.meta.env.VITE_APPLICATION_DEFAULT_MODE === 'mock'
      ? ServiceMode.Mock
      : ServiceMode.Rpc,
};

export const applicationSlice = createSlice({
  initialState,
  name: 'application',
  reducers: {
    setMode: (state) => {
      state.mode =
        state.mode === ServiceMode.Rpc ? ServiceMode.Mock : ServiceMode.Rpc;
    },
  },
});

export const { setMode } = applicationSlice.actions;

export default applicationSlice.reducer;
