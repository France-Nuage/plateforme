import { Instance, InstanceFormValue, InstanceStatus } from '@france-nuage/sdk';
import { PayloadAction, createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { ExtraArgument } from '@/store';

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

export const removeInstance = createAsyncThunk<
  string,
  string,
  { extra: ExtraArgument }
>('instances/remove', (id, { extra }) =>
  extra.services.instance.remove(id).then(() => id),
);

export const startInstance = createAsyncThunk<
  string,
  string,
  { extra: ExtraArgument }
>('instances/start', (id, { extra }) =>
  extra.services.instance.start(id).then(() => id),
);

export const stopInstance = createAsyncThunk<
  string,
  string,
  { extra: ExtraArgument }
>('instances/stop', (id, { extra }) =>
  extra.services.instance.stop(id).then(() => id),
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
    builder.addCase(removeInstance.fulfilled, (state, action) => {
      state.instances.find(
        (instance) => instance.id === action.payload,
      )!.status = InstanceStatus.Terminated;
    });
    builder.addCase(removeInstance.pending, (state, action) => {
      const instance = state.instances.find(
        (instance) => instance.id === action.payload,
      );

      if (instance) {
        instance.status = InstanceStatus.Deprovisionning;
      }
    });
    builder.addCase(startInstance.fulfilled, (state, action) => {
      state.instances.find(
        (instance) => instance.id === action.payload,
      )!.status = InstanceStatus.Running;
    });
    builder.addCase(startInstance.pending, (state, action) => {
      const instance = state.instances.find(
        (instance) => instance.id === action.payload,
      );
      if (instance) {
        instance.status = InstanceStatus.Staging;
      }
    });
    builder.addCase(startInstance.rejected, (state, action) => {
      console.log('my action has been rejected', state, action);
    });
    builder.addCase(stopInstance.fulfilled, (state, action) => {
      state.instances.find(
        (instance) => instance.id === action.payload,
      )!.status = InstanceStatus.Stopped;
    });
    builder.addCase(stopInstance.pending, (state, action) => {
      const instance = state.instances.find(
        (instance) => instance.id === action.payload,
      );

      if (instance) {
        instance.status = InstanceStatus.Stopping;
      }
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
