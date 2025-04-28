import { configureStore } from "@reduxjs/toolkit";
import applicationReducer from "@/features/application.slice";
import instancesReducer from "@/features/instances.slice";
import hypervisorsReducer from "@/features/hypervisors.slice";

export const store = configureStore({
  reducer: {
    application: applicationReducer,
    instances: instancesReducer,
    hypervisors: hypervisorsReducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
export type AppStore = typeof store;
