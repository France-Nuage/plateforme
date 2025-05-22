import { configureStore } from "@reduxjs/toolkit";
import { applicationSlice, authenticationSlice, hypervisorsSlice, instancesSlice } from "@/features";
import { createWrapper } from "next-redux-wrapper";

const makeStore = () =>
  configureStore({
    reducer: {
      [applicationSlice.name]: applicationSlice.reducer,
      [authenticationSlice.name]: authenticationSlice.reducer,
      [hypervisorsSlice.name]: hypervisorsSlice.reducer,
      [instancesSlice.name]: instancesSlice.reducer,
    },
  });

export type AppStore = ReturnType<typeof makeStore>;
export type AppState = ReturnType<AppStore["getState"]>;
export type AppDispatch = AppStore["dispatch"];

export const wrapper = createWrapper<AppStore>(makeStore);
