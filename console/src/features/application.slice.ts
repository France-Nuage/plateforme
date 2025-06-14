import { PayloadAction, createSlice } from '@reduxjs/toolkit';

import { Project } from '@/generated/rpc/resources';
import { Organization, ServiceMode } from '@/types';

export type ApplicationState = {
  activeOrganization?: Organization;
  activeProject?: Project;
  mode: ServiceMode;
};

const initialState: ApplicationState = {
  activeOrganization: undefined,
  activeProject: undefined,
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
    /**
     * Set the active organization.
     */
    setActiveOrganization: (
      state,
      action: PayloadAction<Organization | undefined>,
    ) => {
      state.activeOrganization = action.payload;
    },
    /**
     * Set the active project.
     */
    setActiveProject: (state, action: PayloadAction<Project | undefined>) => {
      state.activeProject = action.payload;
    },
    /**
     * Set the application mode.
     */
    setMode: (state) => {
      state.mode =
        state.mode === ServiceMode.Rpc ? ServiceMode.Mock : ServiceMode.Rpc;
    },
  },
});

export const { setActiveOrganization, setActiveProject, setMode } =
  applicationSlice.actions;

export default applicationSlice.reducer;
