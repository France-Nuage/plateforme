import {
  CreateKubernetesClusterInput,
  CreateKubernetesLabelInput,
  KubernetesCluster,
  KubernetesLabel,
  ManagedServiceRef,
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
 * Fetch every label of the platform registry (attached or not).
 */
export const fetchKubernetesLabels = createAsyncThunk<
  KubernetesLabel[],
  void,
  { extra: ExtraArgument }
>('kubernetesClusters/fetchLabels', (_, { extra }) =>
  extra.services.kubernetesCluster.listLabels(),
);

/**
 * Create a new label in the registry (without attaching it to a cluster).
 */
export const createKubernetesLabel = createAsyncThunk<
  KubernetesLabel,
  CreateKubernetesLabelInput,
  { extra: ExtraArgument }
>('kubernetesClusters/createLabel', (data, { extra }) =>
  extra.services.kubernetesCluster.createLabel(data),
);

/**
 * Delete a label from the registry. The backend cascades the deletion to
 * every cluster attachment; the reducer mirrors that cascade locally.
 *
 * Returns the deleted label's ID so the slice can prune it from the state.
 */
export const deleteKubernetesLabel = createAsyncThunk<
  string,
  string,
  { extra: ExtraArgument }
>('kubernetesClusters/deleteLabel', (labelId, { extra }) =>
  extra.services.kubernetesCluster.deleteLabel(labelId).then(() => labelId),
);

/**
 * Attach a registry label to a cluster, then refetch the cluster so the
 * state reflects the authoritative label list (ordering included).
 */
export const attachKubernetesClusterLabel = createAsyncThunk<
  KubernetesCluster,
  { clusterId: string; labelId: string },
  { extra: ExtraArgument }
>('kubernetesClusters/attachLabel', ({ clusterId, labelId }, { extra }) =>
  extra.services.kubernetesCluster
    .attachClusterLabel(clusterId, labelId)
    .then(() => extra.services.kubernetesCluster.getCluster(clusterId)),
);

/**
 * Detach a label from a cluster, then refetch the cluster.
 *
 * Detaching is always allowed, even when a managed service's deploy_target
 * requires the label: placement happens at deployment time only, so running
 * instances are unaffected. The UI surfaces the impacted services through
 * `fetchServicesReferencingLabel` before dispatching this thunk.
 */
export const detachKubernetesClusterLabel = createAsyncThunk<
  KubernetesCluster,
  { clusterId: string; labelId: string },
  { extra: ExtraArgument }
>('kubernetesClusters/detachLabel', ({ clusterId, labelId }, { extra }) =>
  extra.services.kubernetesCluster
    .detachClusterLabel(clusterId, labelId)
    .then(() => extra.services.kubernetesCluster.getCluster(clusterId)),
);

/**
 * List the active managed services whose deploy_target requires a label.
 *
 * Used by the detach/delete confirmation dialogs to warn the operator; the
 * result is consumed via `.unwrap()` and never stored in the slice.
 */
export const fetchServicesReferencingLabel = createAsyncThunk<
  ManagedServiceRef[],
  string,
  { extra: ExtraArgument }
>('kubernetesClusters/fetchServicesReferencingLabel', (labelId, { extra }) =>
  extra.services.kubernetesCluster.listServicesReferencingLabel(labelId),
);

/**
 * Shape of the Kubernetes clusters feature slice state.
 */
export type KubernetesClustersState = {
  clusters: KubernetesCluster[];
  currentCluster: KubernetesCluster | undefined;
  /** Platform-wide label registry, ordered by key then value. */
  labels: KubernetesLabel[];
  loading: boolean;
};

const initialState: KubernetesClustersState = {
  clusters: [],
  currentCluster: undefined,
  labels: [],
  loading: false,
};

/**
 * Mirrors the backend ordering (CITEXT `ORDER BY key, value`) so labels
 * created client-side slot in at the right position.
 */
function compareLabels(a: KubernetesLabel, b: KubernetesLabel): number {
  return (
    a.key.toLowerCase().localeCompare(b.key.toLowerCase()) ||
    a.value.toLowerCase().localeCompare(b.value.toLowerCase())
  );
}

/**
 * Replaces a cluster in the list and the current cluster (when it matches)
 * with a fresh server-side snapshot.
 */
function replaceCluster(
  state: KubernetesClustersState,
  cluster: KubernetesCluster,
): void {
  const index = state.clusters.findIndex((entry) => entry.id === cluster.id);
  if (index !== -1) {
    state.clusters[index] = cluster;
  }
  if (state.currentCluster?.id === cluster.id) {
    state.currentCluster = cluster;
  }
}

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
      replaceCluster(state, action.payload);
    });

    builder.addCase(fetchKubernetesLabels.fulfilled, (state, action) => {
      state.labels = action.payload;
    });

    builder.addCase(createKubernetesLabel.fulfilled, (state, action) => {
      state.labels.push(action.payload);
      state.labels.sort(compareLabels);
    });

    builder.addCase(deleteKubernetesLabel.fulfilled, (state, action) => {
      state.labels = state.labels.filter(
        (label) => label.id !== action.payload,
      );
      // Mirror the backend ON DELETE CASCADE on cluster attachments.
      for (const cluster of state.clusters) {
        cluster.labels = cluster.labels.filter(
          (label) => label.id !== action.payload,
        );
      }
      if (state.currentCluster) {
        state.currentCluster.labels = state.currentCluster.labels.filter(
          (label) => label.id !== action.payload,
        );
      }
    });

    builder.addCase(attachKubernetesClusterLabel.fulfilled, (state, action) => {
      replaceCluster(state, action.payload);
    });

    builder.addCase(detachKubernetesClusterLabel.fulfilled, (state, action) => {
      replaceCluster(state, action.payload);
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
