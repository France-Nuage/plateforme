import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { ExtraArgument } from '@/store';
import { Datacenter, ZeroTrustNetwork, ZeroTrustNetworkType } from '@/types';

/**
 * Fetch all datacenters
 */
export const fetchAllDatacenters = createAsyncThunk<
  Datacenter[],
  void,
  { extra: ExtraArgument }
>('resources/fetchAllDatacenters', async (_, { extra }) =>
  extra.services.datacenter.list(),
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
  datacenters: Datacenter[];
  zeroTrustNetworkTypes: ZeroTrustNetworkType[];
  zeroTrustNetworks: ZeroTrustNetwork[];
};

/**
 * The resources slice initial state.
 */
const initialState: InfrastructureState = {
  datacenters: [],
  zeroTrustNetworks: [],
  zeroTrustNetworkTypes: [],
};

/**
 * The resources slice.
 */
export const infrastructureSlice = createSlice({
  extraReducers: (builder) => {
    builder
      .addCase(fetchAllDatacenters.fulfilled, (state, action) => {
        state.datacenters = action.payload;
      })
      .addCase(fetchAllZeroTrustNetworkTypes.fulfilled, (state, action) => {
        state.zeroTrustNetworkTypes = action.payload;
      })
      .addCase(fetchAllZeroTrustNetworks.fulfilled, (state, action) => {
        state.zeroTrustNetworks = action.payload;
      });
  },
  initialState,
  name: 'infrastructure',
  reducers: {},
});

export default infrastructureSlice;
