import { services } from "@/services";
import { RootState } from "@/store";
import { Instance, InstanceFormValue } from "@/types";
import { createAsyncThunk, createSlice, PayloadAction } from "@reduxjs/toolkit";

export const fetchAllInstances = createAsyncThunk<
  Instance[],
  void,
  { state: RootState }
>("instances/fetchAll", async (_, { getState }) => {
  const mode = getState().application.mode;
  return await services[mode].instance.list();
});

export const createInstance = createAsyncThunk<
  Instance,
  InstanceFormValue,
  { state: RootState }
>("instances/create", (data, { getState }) =>
  services[getState().application.mode].instance.create(data),
);

export type InstancesState = {
  instances: Instance[];
};

const initialState: InstancesState = {
  instances: [],
};

export const instancesSlice = createSlice({
  name: "instances",
  initialState,
  reducers: {
    addInstance: (state, action: PayloadAction<Instance>) => {
      state.instances.push(action.payload);
    },
  },
  extraReducers: (builder) => {
    builder.addCase(fetchAllInstances.fulfilled, (state, action) => {
      state.instances = action.payload;
    });
    builder.addCase(createInstance.fulfilled, (state, action) => {
      state.instances.push(action.payload);
    });
  },
});

export const { addInstance } = instancesSlice.actions;

export default instancesSlice;
