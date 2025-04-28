import { services } from "@/services";
import { RootState } from "@/store";
import { Hypervisor, HypervisorFormValue } from "@/types";
import { createAsyncThunk, createSlice, PayloadAction } from "@reduxjs/toolkit";

export const fetchAllHypervisors = createAsyncThunk<
  Hypervisor[],
  void,
  { state: RootState }
>("hypervisors/fetchAll", async (_, { getState }) =>
  services[getState().application.mode].hypervisor.list(),
);

export const registerHypervisor = createAsyncThunk<
  Hypervisor,
  HypervisorFormValue,
  { state: RootState }
>("hypervisors/register", (data, { getState }) =>
  services[getState().application.mode].hypervisor.register(data),
);

export interface HypervisorsState {
  hypervisors: Hypervisor[];
}

const initialState: HypervisorsState = {
  hypervisors: [],
};

export const hypervisorsSlice = createSlice({
  name: "hypervisors",
  initialState,
  reducers: {
    addInstance: (state, action: PayloadAction<Hypervisor>) => {
      state.hypervisors = [...state.hypervisors, action.payload];
    },
  },
  extraReducers: (builder) => {
    builder.addCase(fetchAllHypervisors.fulfilled, (state, action) => {
      state.hypervisors = action.payload;
    });
  },
});

export const { addInstance: addHypervisor } = hypervisorsSlice.actions;

export default hypervisorsSlice.reducer;
