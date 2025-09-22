import { configureStore } from '@reduxjs/toolkit';

import {
  applicationSlice,
  authenticationSlice,
  hypervisorsSlice,
  infrastructureSlice,
  instancesSlice,
  resourcesSlice,
} from '@/features';
import { configureServices } from '@/services';

const extraArgument = {
  get services() {
    const { dispatch } = store;
    const state = store.getState();

    return configureServices({ dispatch, state })[state.application.mode];
  },
};

export const store = configureStore({
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      thunk: {
        extraArgument,
      },
    }),
  reducer: {
    [applicationSlice.name]: applicationSlice.reducer,
    [authenticationSlice.name]: authenticationSlice.reducer,
    [hypervisorsSlice.name]: hypervisorsSlice.reducer,
    [infrastructureSlice.name]: infrastructureSlice.reducer,
    [instancesSlice.name]: instancesSlice.reducer,
    [resourcesSlice.name]: resourcesSlice.reducer,
  },
});

export type AppDispatch = typeof store.dispatch;
export type ExtraArgument = typeof extraArgument;
export type AppState = ReturnType<typeof store.getState>;
export type AppStore = { state: AppState; dispatch: AppDispatch };
