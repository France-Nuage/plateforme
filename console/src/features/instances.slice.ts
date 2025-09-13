import { PayloadAction, createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { ExtraArgument } from '@/store';
import { Instance, InstanceFormValue } from '@/types';

export const fetchAllInstances = createAsyncThunk<
  Instance[],
  void,
  { extra: ExtraArgument }
>('instances/fetchAll', async (_, { extra }) => {
  return await extra.services.instance.list();
});

export const createInstance = createAsyncThunk<
  Instance,
  InstanceFormValue,
  { extra: ExtraArgument }
>('instances/create', (data, { extra }) =>
  extra.services.instance.create(data),
);

export type InstancesState = {
  instances: Instance[];
};

const initialState: InstancesState = {
  instances: [],
};

export const instancesSlice = createSlice({
  extraReducers: (builder) => {
    builder.addCase(fetchAllInstances.fulfilled, (state, action) => {
      state.instances = action.payload;
    });
    builder.addCase(createInstance.fulfilled, (state, action) => {
      state.instances.push(action.payload);
    });
  },
  initialState,
  name: 'instances',
  reducers: {
    addInstance: (state, action: PayloadAction<Instance>) => {
      state.instances.push(action.payload);
    },
  },
});

export const { addInstance } = instancesSlice.actions;

export default instancesSlice;
