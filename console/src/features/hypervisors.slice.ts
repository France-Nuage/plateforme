import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { services } from '@/services';
import { RootState } from '@/store';
import { Hypervisor, HypervisorFormValue } from '@/types';

export const fetchAllHypervisors = createAsyncThunk<
  Hypervisor[],
  void,
  { state: RootState }
>('hypervisors/fetchAll', async (_, { getState }) =>
  services[getState().application.mode].hypervisor.list(),
);

export const registerHypervisor = createAsyncThunk<
  Hypervisor,
  HypervisorFormValue,
  { state: RootState }
>('hypervisors/register', (data, { getState }) =>
  services[getState().application.mode].hypervisor.register(data),
);

export type HypervisorsState = {
  hypervisors: Hypervisor[];
};

const initialState: HypervisorsState = {
  hypervisors: [],
};

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
