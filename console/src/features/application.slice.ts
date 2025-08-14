import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { Project } from '@/generated/rpc/resources';
import { RootState } from '@/store';
import { Organization, ServiceMode } from '@/types';

/**
 * The slice state type.
 */
export type ApplicationState = {
  activeOrganization: Organization | undefined;
  activeProject: Project | undefined;
  mode: ServiceMode;
};

/**
 * The slice initial state.
 */
const initialState: ApplicationState = {
  activeOrganization: undefined,
  activeProject: undefined,
  mode: window.location.pathname.startsWith('/plasmic-host')
    ? ServiceMode.Mock
    : import.meta.env.VITE_APPLICATION_DEFAULT_MODE === 'mock'
      ? ServiceMode.Mock
      : ServiceMode.Rpc,
};

/**
 * Set the active organization.
 */
export const setActiveOrganization = createAsyncThunk<
  { organization: Organization; project: Project },
  Organization,
  { state: RootState }
>('application/setActiveOrganization', (organization, { getState }) => {
  // Retrieve a default project for the new active organization
  const state = getState();
  const project = state.resources.projects.find(
    (project) => project.organizationId === organization.id,
  );

  // Throw an error if no active project could be found
  if (!project) {
    throw new Error(
      `Could not find any project for organization ${organization.id}`,
    );
  }

  return {
    organization,
    project,
  };
});

/**
 * Set the active project.
 */
export const setActiveProject = createAsyncThunk<
  { organization: Organization; project: Project },
  Project,
  { state: RootState }
>('application/setActiveProject', (project, { getState }) => {
  // Retrieve a default project for the new active organization
  const state = getState();
  const organization = state.resources.organizations.find(
    (organization) => project.organizationId === organization.id,
  );

  // Throw an error if the organization matching the given project could not be found
  if (!organization) {
    throw new Error(
      `Could not find the organization matching the project ${project.id}`,
    );
  }

  return {
    organization,
    project: project,
  };
});

/**
 * The application slice.
 */
export const applicationSlice = createSlice({
  extraReducers: (builder) => {
    builder
      .addCase(setActiveOrganization.fulfilled, (state, action) => {
        state.activeOrganization = action.payload.organization;
        state.activeProject = action.payload.project;
      })
      .addCase(setActiveProject.fulfilled, (state, action) => {
        state.activeOrganization = action.payload.organization;
        state.activeProject = action.payload.project;
      });
  },
  initialState,
  name: 'application',
  reducers: {
    /**
     * Set the application mode.
     */
    setMode: (state) => {
      state.mode =
        state.mode === ServiceMode.Rpc ? ServiceMode.Mock : ServiceMode.Rpc;
    },
  },
});

export const { setMode } = applicationSlice.actions;

export default applicationSlice.reducer;
