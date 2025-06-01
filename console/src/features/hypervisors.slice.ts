import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { services } from '@/services';
import { RootState } from '@/store';
import { Hypervisor, HypervisorFormValue } from '@/types';

/**
 * Fetch all hypervisors.
 */
export const fetchAllHypervisors = createAsyncThunk<
  Hypervisor[],
  void,
  { state: RootState }
>('hypervisors/fetchAll', async (_, { getState }) =>
  services[getState().application.mode].hypervisor.list(),
);

/**
 * Register a new hypervisor.
 */
export const registerHypervisor = createAsyncThunk<
  Hypervisor,
  HypervisorFormValue,
  { state: RootState }
>('hypervisors/register', (data, { getState }) =>
  services[getState().application.mode].hypervisor.register(data),
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
