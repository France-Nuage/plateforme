import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { Datacenter } from '@/generated/rpc/infrastructure';
import { services } from '@/services';
import { RootState } from '@/store';
import { ZeroTrustNetwork, ZeroTrustNetworkType } from '@/types';

/**
 * Fetch all datacenters
 */
export const fetchAllDatacenters = createAsyncThunk<
  Datacenter[],
  void,
  { state: RootState }
>('resources/fetchAllDatacenters', async (_, { getState }) =>
  services[getState().application.mode].datacenter.list(),
);

/**
 * Fetch all zero trust network types
 */
export const fetchAllZeroTrustNetworkTypes = createAsyncThunk<
  ZeroTrustNetworkType[],
  void,
  { state: RootState }
>('resources/fetchAllZeroTrustNetworkTypes', async (_, { getState }) =>
  services[getState().application.mode].zeroTrustNetworkType.list(),
);

/**
 * Fetch all projects
 */
export const fetchAllZeroTrustNetworks = createAsyncThunk<
  ZeroTrustNetwork[],
  void,
  { state: RootState }
>('resources/fetchAllZeroTrustNetworks', async (_, { getState }) =>
  services[getState().application.mode].zeroTrustNetwork.list(),
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
