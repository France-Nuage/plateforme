import { services } from "@/services";
import { RootState } from "@/store";
import { Organization, Project } from "@/types";
import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";

/**
 * Fetch all organizations
 */
export const fetchAllOrganizations = createAsyncThunk<
  Organization[],
  void,
  { state: RootState }
>('resources/fetchAllOrganizations', async (_, { getState }) =>
  services[getState().application.mode].organization.list(),
);

/**
 * Fetch all projects
 */
export const fetchAllProjects = createAsyncThunk<
  Project[],
  void,
  { state: RootState }
>('resources/fetchAllProjects', async (_, { getState }) =>
  services[getState().application.mode].project.list(),
);

/**
 * The resources slice state shape.
 */
export type ResourcesState = {
  organizations: Organization[];
  projects: Project[];
}

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
      }).addCase(fetchAllProjects.fulfilled, (state, action) => {
        state.projects = action.payload;
      });
  },
  initialState,
  name: 'resources',
  reducers: {},
})

export default resourcesSlice;

