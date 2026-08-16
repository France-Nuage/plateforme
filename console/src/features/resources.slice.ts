import { Organization, Project } from '@france-nuage/sdk';
import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { ExtraArgument } from '@/store';

import { logout } from './authentication.slice';

/**
 * Fetch all organizations
 */
export const fetchAllOrganizations = createAsyncThunk<
  Organization[],
  void,
  { extra: ExtraArgument }
>('resources/fetchAllOrganizations', async (_, { extra }) =>
  extra.services.organization.list(),
);

/**
 * Fetch all projects
 */
export const fetchAllProjects = createAsyncThunk<
  Project[],
  void,
  { extra: ExtraArgument }
>('resources/fetchAllProjects', async (_, { extra }) =>
  extra.services.project.list(),
);

/**
 * The resources slice state shape.
 */
export type ResourcesState = {
  organizations: Organization[];
  /** Whether organizations have been fetched from the API at least once. */
  organizationsLoaded: boolean;
  /**
   * Whether the last organizations fetch failed. Distinct from "not yet loaded"
   * so the guard can show an error state with a retry instead of spinning
   * forever when the org-list call rejects with an unrecoverable error (i.e.
   * anything the gRPC auth interceptor does not recover, such as a backend
   * failure).
   */
  organizationsError: boolean;
  projects: Project[];
};

/**
 * The resources slice initial state.
 */
const initialState: ResourcesState = {
  organizations: [],
  organizationsError: false,
  organizationsLoaded: false,
  projects: [],
};

/**
 * The resources slice.
 */
export const resourcesSlice = createSlice({
  extraReducers: (builder) => {
    builder
      .addCase(fetchAllOrganizations.pending, (state) => {
        // Clear any previous failure so a retry shows the spinner again.
        state.organizationsError = false;
      })
      .addCase(fetchAllOrganizations.fulfilled, (state, action) => {
        state.organizations = action.payload;
        state.organizationsLoaded = true;
        state.organizationsError = false;
      })
      .addCase(fetchAllOrganizations.rejected, (state) => {
        // Stop spinning: record the failure so the guard renders an error state
        // with a retry instead of an infinite loading spinner.
        state.organizationsError = true;
      })
      .addCase(fetchAllProjects.fulfilled, (state, action) => {
        state.projects = action.payload;
      })
      .addCase(logout.fulfilled, (state) => {
        state.organizations = [];
        state.organizationsLoaded = false;
        state.organizationsError = false;
        state.projects = [];
      });
  },
  initialState,
  name: 'resources',
  reducers: {},
});

export default resourcesSlice;
