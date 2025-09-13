import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { ExtraArgument } from '@/store';
import { Organization, Project } from '@/types';

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
  projects: Project[];
};

/**
 * The resources slice initial state.
 */
const initialState: ResourcesState = {
  organizations: [],
  projects: [],
};

/**
 * The resources slice.
 */
export const resourcesSlice = createSlice({
  extraReducers: (builder) => {
    builder
      .addCase(fetchAllOrganizations.fulfilled, (state, action) => {
        state.organizations = action.payload;
      })
      .addCase(fetchAllProjects.fulfilled, (state, action) => {
        state.projects = action.payload;
      })
      .addCase(logout.fulfilled, (state) => {
        state.organizations = [];
        state.projects = [];
      });
  },
  initialState,
  name: 'resources',
  reducers: {},
});

export default resourcesSlice;
