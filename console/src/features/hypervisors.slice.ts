import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { ExtraArgument } from '@/store';
import { Hypervisor, HypervisorFormValue } from '@/types';

/**
 * Fetch all hypervisors.
 */
export const fetchAllHypervisors = createAsyncThunk<
  Hypervisor[],
  void,
  { extra: ExtraArgument }
>('hypervisors/fetchAll', async (_, { extra }) =>
  extra.services.hypervisor.list(),
);

/**
 * Register a new hypervisor.
 */
export const registerHypervisor = createAsyncThunk<
  Hypervisor,
  HypervisorFormValue,
  { extra: ExtraArgument }
>('hypervisors/register', (data, { extra }) =>
  extra.services.hypervisor.register(data),
);

/**
 * The hypervisors slice state shape.
 */
export type HypervisorsState = {
  hypervisors: Hypervisor[];
};

/**
 * The hypervisors slice initial state.
 */
const initialState: HypervisorsState = {
  hypervisors: [],
};

/**
 * The hypervisors slice.
 */
export const hypervisorsSlice = createSlice({
  extraReducers: (builder) => {
    builder
      .addCase(fetchAllHypervisors.fulfilled, (state, action) => {
        state.hypervisors = action.payload;
      })
      .addCase(registerHypervisor.fulfilled, (state, action) => {
        state.hypervisors.push(action.payload);
      });
  },
  initialState,
  name: 'hypervisors',
  reducers: {},
});

export default hypervisorsSlice;
