import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { services } from '@/services';
import { RootState } from '@/store';
import { ZeroTrustNetwork, ZeroTrustNetworkType } from '@/types';

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
  zeroTrustNetworkTypes: ZeroTrustNetworkType[];
  zeroTrustNetworks: ZeroTrustNetwork[];
};

/**
 * The resources slice initial state.
 */
const initialState: InfrastructureState = {
  zeroTrustNetworks: [],
  zeroTrustNetworkTypes: [],
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
      });
  },
  initialState,
  name: 'infrastructure',
  reducers: {},
});

export default infrastructureSlice;
