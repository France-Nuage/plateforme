import { Zone } from '@france-nuage/sdk';
import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { ExtraArgument } from '@/store';

/**
 * Fetch all zones
 */
export const fetchAllZones = createAsyncThunk<
  Zone[],
  void,
  { extra: ExtraArgument }
>('resources/fetchAllZones', async (_, { extra }) =>
  extra.services.zone.list(),
);

/**
 * The resources slice state shape.
 */
export type InfrastructureState = {
  zones: Zone[];
};

/**
 * The resources slice initial state.
 */
const initialState: InfrastructureState = {
  zones: [],
};

/**
 * The resources slice.
 */
export const infrastructureSlice = createSlice({
  extraReducers: (builder) => {
    builder.addCase(fetchAllZones.fulfilled, (state, action) => {
      state.zones = action.payload;
    });
  },
  initialState,
  name: 'infrastructure',
  reducers: {},
});

export default infrastructureSlice;
