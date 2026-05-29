import {
  CreateManagedInstanceInput,
  ManagedInstanceStatus,
  ManagedService,
  ManagedServiceInstance,
  ManagedServiceVersion,
  UpgradeManagedInstanceInput,
} from '@france-nuage/sdk';
import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { ExtraArgument } from '@/store';

export const fetchAllManagedServices = createAsyncThunk<
  ManagedService[],
  void,
  { extra: ExtraArgument }
>('managedServices/fetchAll', (_, { extra }) =>
  extra.services.managedService.listServices(),
);

export const fetchManagedService = createAsyncThunk<
  ManagedService,
  string,
  { extra: ExtraArgument }
>('managedServices/fetchOne', (slug, { extra }) =>
  extra.services.managedService.getService(slug),
);

export const fetchManagedServiceVersions = createAsyncThunk<
  ManagedServiceVersion[],
  string,
  { extra: ExtraArgument }
>('managedServices/fetchVersions', (serviceSlug, { extra }) =>
  extra.services.managedService.listVersions(serviceSlug),
);

export const fetchManagedInstances = createAsyncThunk<
  ManagedServiceInstance[],
  string,
  { extra: ExtraArgument }
>('managedServices/fetchInstances', (projectId, { extra }) =>
  extra.services.managedService.listInstances(projectId),
);

export const fetchManagedInstance = createAsyncThunk<
  ManagedServiceInstance,
  string,
  { extra: ExtraArgument }
>('managedServices/fetchInstance', (instanceId, { extra }) =>
  extra.services.managedService.getInstance(instanceId),
);

export const createManagedInstance = createAsyncThunk<
  ManagedServiceInstance,
  CreateManagedInstanceInput,
  { extra: ExtraArgument }
>('managedServices/createInstance', (data, { extra }) =>
  extra.services.managedService.createInstance(data),
);

export const upgradeManagedInstance = createAsyncThunk<
  ManagedServiceInstance,
  UpgradeManagedInstanceInput,
  { extra: ExtraArgument }
>('managedServices/upgradeInstance', (data, { extra }) =>
  extra.services.managedService.upgradeInstance(data),
);

export const deleteManagedInstance = createAsyncThunk<
  string,
  string,
  { extra: ExtraArgument }
>('managedServices/deleteInstance', (instanceId, { extra }) =>
  extra.services.managedService
    .deleteInstance(instanceId)
    .then(() => instanceId),
);

export type ManagedServicesState = {
  currentInstance: ManagedServiceInstance | undefined;
  currentService: ManagedService | undefined;
  instances: ManagedServiceInstance[];
  loading: boolean;
  services: ManagedService[];
  versions: ManagedServiceVersion[];
};

const initialState: ManagedServicesState = {
  currentInstance: undefined,
  currentService: undefined,
  instances: [],
  loading: false,
  services: [],
  versions: [],
};

export const managedServicesSlice = createSlice({
  extraReducers: (builder) => {
    builder.addCase(fetchAllManagedServices.pending, (state) => {
      state.loading = true;
    });
    builder.addCase(fetchAllManagedServices.fulfilled, (state, action) => {
      state.services = action.payload;
      state.loading = false;
    });
    builder.addCase(fetchAllManagedServices.rejected, (state) => {
      state.loading = false;
    });

    builder.addCase(fetchManagedService.fulfilled, (state, action) => {
      state.currentService = action.payload;
    });

    builder.addCase(fetchManagedServiceVersions.fulfilled, (state, action) => {
      state.versions = action.payload;
    });

    builder.addCase(fetchManagedInstances.fulfilled, (state, action) => {
      state.instances = action.payload;
    });

    builder.addCase(fetchManagedInstance.fulfilled, (state, action) => {
      state.currentInstance = action.payload;
    });

    builder.addCase(createManagedInstance.fulfilled, (state, action) => {
      state.instances.push(action.payload);
    });

    builder.addCase(upgradeManagedInstance.fulfilled, (state, action) => {
      const index = state.instances.findIndex(
        (instance) => instance.id === action.payload.id,
      );
      if (index !== -1) {
        state.instances[index] = action.payload;
      }
      if (state.currentInstance?.id === action.payload.id) {
        state.currentInstance = action.payload;
      }
    });

    builder.addCase(deleteManagedInstance.pending, (state, action) => {
      const instance = state.instances.find(
        (instance) => instance.id === action.meta.arg,
      );
      if (instance) {
        instance.status = ManagedInstanceStatus.Deleting;
      }
    });
    builder.addCase(deleteManagedInstance.fulfilled, (state, action) => {
      state.instances = state.instances.filter(
        (instance) => instance.id !== action.payload,
      );
    });
  },
  initialState,
  name: 'managedServices',
  reducers: {},
});

export default managedServicesSlice;
