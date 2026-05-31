import {
  CreateKubernetesClusterInput,
  KubernetesCluster,
  UpdateKubernetesClusterInput,
} from '@france-nuage/sdk';
import { createAsyncThunk, createSlice } from '@reduxjs/toolkit';

import { ExtraArgument } from '@/store';

/**
 * Fetch all Kubernetes clusters registered on the platform.
 */
export const fetchAllKubernetesClusters = createAsyncThunk<
  KubernetesCluster[],
  void,
  { extra: ExtraArgument }
>('kubernetesClusters/fetchAll', (_, { extra }) =>
  extra.services.kubernetesCluster.listClusters(),
);

/**
 * Fetch a single Kubernetes cluster by its ID.
 */
export const fetchKubernetesCluster = createAsyncThunk<
  KubernetesCluster,
  string,
  { extra: ExtraArgument }
>('kubernetesClusters/fetchOne', (clusterId, { extra }) =>
  extra.services.kubernetesCluster.getCluster(clusterId),
);

/**
 * Create a new Kubernetes cluster.
 *
 * The backend performs a synchronous reachability check against the cluster
 * and rejects with a FAILED_PRECONDITION error when the cluster is unreachable.
 * Callers should handle that rejection and surface the error message.
 */
export const createKubernetesCluster = createAsyncThunk<
  KubernetesCluster,
  CreateKubernetesClusterInput,
  { extra: ExtraArgument }
>('kubernetesClusters/create', (data, { extra }) =>
  extra.services.kubernetesCluster.createCluster(data),
);

/**
 * Update an existing Kubernetes cluster.
 *
 * When a kubeconfig is provided, the backend performs the same synchronous
 * reachability check as on creation.
 */
export const updateKubernetesCluster = createAsyncThunk<
  KubernetesCluster,
  UpdateKubernetesClusterInput,
  { extra: ExtraArgument }
>('kubernetesClusters/update', (data, { extra }) =>
  extra.services.kubernetesCluster.updateCluster(data),
);

/**
 * Delete a Kubernetes cluster by its ID.
 *
 * Returns the deleted cluster's ID so the slice can remove it from the list.
 */
export const deleteKubernetesCluster = createAsyncThunk<
  string,
  string,
  { extra: ExtraArgument }
>('kubernetesClusters/delete', (clusterId, { extra }) =>
  extra.services.kubernetesCluster
    .deleteCluster(clusterId)
    .then(() => clusterId),
);

/**
 * Shape of the Kubernetes clusters feature slice state.
 */
export type KubernetesClustersState = {
  clusters: KubernetesCluster[];
  currentCluster: KubernetesCluster | undefined;
  loading: boolean;
};

const initialState: KubernetesClustersState = {
  clusters: [],
  currentCluster: undefined,
  loading: false,
};

/**
 * The Kubernetes clusters feature slice.
 *
 * Stores the list of platform clusters and the currently viewed cluster.
 * This is a platform-admin feature; the server enforces authorization via
 * the user's `is_admin` flag.
 */
export const kubernetesClustersSlice = createSlice({
  extraReducers: (builder) => {
    builder.addCase(fetchAllKubernetesClusters.pending, (state) => {
      state.loading = true;
    });
    builder.addCase(fetchAllKubernetesClusters.fulfilled, (state, action) => {
      state.clusters = action.payload;
      state.loading = false;
    });
    builder.addCase(fetchAllKubernetesClusters.rejected, (state) => {
      state.loading = false;
    });

    builder.addCase(fetchKubernetesCluster.fulfilled, (state, action) => {
      state.currentCluster = action.payload;
    });

    builder.addCase(createKubernetesCluster.fulfilled, (state, action) => {
      state.clusters.push(action.payload);
    });

    builder.addCase(updateKubernetesCluster.fulfilled, (state, action) => {
      const index = state.clusters.findIndex(
        (cluster) => cluster.id === action.payload.id,
      );
      if (index !== -1) {
        state.clusters[index] = action.payload;
      }
      if (state.currentCluster?.id === action.payload.id) {
        state.currentCluster = action.payload;
      }
    });

    builder.addCase(deleteKubernetesCluster.fulfilled, (state, action) => {
      state.clusters = state.clusters.filter(
        (cluster) => cluster.id !== action.payload,
      );
      if (state.currentCluster?.id === action.payload) {
        state.currentCluster = undefined;
      }
    });
  },
  initialState,
  name: 'kubernetesClusters',
  reducers: {
    /**
     * Clears the currently viewed cluster, preventing stale data from appearing
     * while the edit page fetches the real cluster by ID.
     */
    clearCurrentCluster: (state) => {
      state.currentCluster = undefined;
    },
  },
});

export const { clearCurrentCluster } = kubernetesClustersSlice.actions;

export default kubernetesClustersSlice;
