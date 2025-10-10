import {
  ZeroTrustNetwork,
  ZeroTrustNetworkType,
  Zone,
} from '@france-nuage/sdk';
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
 * Fetch all zero trust network types
 */
export const fetchAllZeroTrustNetworkTypes = createAsyncThunk<
  ZeroTrustNetworkType[],
  void,
  { extra: ExtraArgument }
>('resources/fetchAllZeroTrustNetworkTypes', async (_, { extra }) =>
  extra.services.zeroTrustNetworkType.list(),
);

/**
 * Fetch all projects
 */
export const fetchAllZeroTrustNetworks = createAsyncThunk<
  ZeroTrustNetwork[],
  void,
  { extra: ExtraArgument }
>('resources/fetchAllZeroTrustNetworks', async (_, { extra }) =>
  extra.services.zeroTrustNetwork.list(),
);

/**
 * The resources slice state shape.
 */
export type InfrastructureState = {
  zeroTrustNetworkTypes: ZeroTrustNetworkType[];
  zeroTrustNetworks: ZeroTrustNetwork[];
  zones: Zone[];
};

/**
 * The resources slice initial state.
 */
const initialState: InfrastructureState = {
  zeroTrustNetworks: [],
  zeroTrustNetworkTypes: [],
  zones: [],
};

/**
 * The resources slice.
 */
export const infrastructureSlice = createSlice({
  extraReducers: (builder) => {
    builder
      .addCase(fetchAllZeroTrustNetworkTypes.fulfilled, (state, action) => {
        state.zeroTrustNetworkTypes = action.payload;
      })
      .addCase(fetchAllZeroTrustNetworks.fulfilled, (state, action) => {
        state.zeroTrustNetworks = action.payload;
      })
      .addCase(fetchAllZones.fulfilled, (state, action) => {
        state.zones = action.payload;
      });
  },
  initialState,
  name: 'infrastructure',
  reducers: {},
});

export default infrastructureSlice;
