import { configureStore } from '@reduxjs/toolkit';

import {
  applicationSlice,
  authenticationSlice,
  hypervisorsSlice,
  infrastructureSlice,
  instancesSlice,
  resourcesSlice,
} from '@/features';

export const store = configureStore({
  reducer: {
    [applicationSlice.name]: applicationSlice.reducer,
    [authenticationSlice.name]: authenticationSlice.reducer,
    [hypervisorsSlice.name]: hypervisorsSlice.reducer,
    [infrastructureSlice.name]: infrastructureSlice.reducer,
    [instancesSlice.name]: instancesSlice.reducer,
    [resourcesSlice.name]: resourcesSlice.reducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
