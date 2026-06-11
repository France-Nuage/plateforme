import {
  KubernetesCluster,
  KubernetesClusterHealthStatus,
  KubernetesLabel,
} from '@france-nuage/sdk';
import { describe, expect, it } from 'vitest';

import kubernetesClustersSlice, {
  KubernetesClustersState,
  attachKubernetesClusterLabel,
  createKubernetesCluster,
  createKubernetesLabel,
  deleteKubernetesLabel,
  detachKubernetesClusterLabel,
  fetchKubernetesLabels,
} from './kubernetes-clusters.slice';

const reducer = kubernetesClustersSlice.reducer;

function buildLabel(
  id: string,
  key: string,
  value: string,
  system = false,
): KubernetesLabel {
  return {
    createdAt: '2026-06-11T00:00:00.000Z',
    id,
    key,
    system,
    updatedAt: '2026-06-11T00:00:00.000Z',
    value,
  };
}

function buildCluster(
  id: string,
  labels: KubernetesLabel[] = [],
): KubernetesCluster {
  return {
    apiServerUrl: 'https://cluster.example:6443',
    createdAt: '2026-06-11T00:00:00.000Z',
    healthStatus: KubernetesClusterHealthStatus.Healthy,
    id,
    labels,
    name: `cluster-${id}`,
    updatedAt: '2026-06-11T00:00:00.000Z',
  };
}

function stateWith(
  partial: Partial<KubernetesClustersState>,
): KubernetesClustersState {
  return {
    clusters: [],
    currentCluster: undefined,
    labels: [],
    loading: false,
    ...partial,
  };
}

describe('kubernetes labels registry', () => {
  it('stores the fetched labels', () => {
    const labels = [buildLabel('l1', 'availability', 'ft')];
    const state = reducer(
      undefined,
      fetchKubernetesLabels.fulfilled(labels, 'req', undefined),
    );
    expect(state.labels).toEqual(labels);
  });

  it('inserts a created label keeping the key/value ordering', () => {
    const initial = stateWith({
      labels: [
        buildLabel('l1', 'availability', 'ft'),
        buildLabel('l3', 'region', 'eu'),
      ],
    });
    const state = reducer(
      initial,
      createKubernetesLabel.fulfilled(
        buildLabel('l2', 'AVAILABILITY', 'zz'),
        'req',
        { key: 'AVAILABILITY', value: 'zz' },
      ),
    );
    expect(state.labels.map((label) => label.id)).toEqual(['l1', 'l2', 'l3']);
  });

  it('removes a deleted label from the registry and cascades to clusters', () => {
    const doomed = buildLabel('l1', 'availability', 'ft');
    const kept = buildLabel('l2', 'region', 'eu');
    const initial = stateWith({
      clusters: [buildCluster('c1', [doomed, kept])],
      currentCluster: buildCluster('c1', [doomed, kept]),
      labels: [doomed, kept],
    });

    const state = reducer(
      initial,
      deleteKubernetesLabel.fulfilled('l1', 'req', 'l1'),
    );

    expect(state.labels).toEqual([kept]);
    expect(state.clusters[0].labels).toEqual([kept]);
    expect(state.currentCluster?.labels).toEqual([kept]);
  });
});

describe('cluster creation with labels', () => {
  it('stores the created cluster with the labels attached at creation', () => {
    const label = buildLabel('l1', 'availability', 'ft');
    const created = buildCluster('c1', [label]);

    const state = reducer(
      undefined,
      createKubernetesCluster.fulfilled(created, 'req', {
        kubeconfig: 'apiVersion: v1',
        labelIds: ['l1'],
        name: 'cluster-c1',
      }),
    );

    expect(state.clusters).toHaveLength(1);
    expect(state.clusters[0].labels).toEqual([label]);
  });
});

describe('cluster label attachments', () => {
  const label = buildLabel('l1', 'availability', 'ft');

  it('replaces the cluster snapshot after an attach', () => {
    const initial = stateWith({
      clusters: [buildCluster('c1'), buildCluster('c2')],
      currentCluster: buildCluster('c1'),
    });
    const refreshed = buildCluster('c1', [label]);

    const state = reducer(
      initial,
      attachKubernetesClusterLabel.fulfilled(refreshed, 'req', {
        clusterId: 'c1',
        labelId: 'l1',
      }),
    );

    expect(state.clusters[0].labels).toEqual([label]);
    expect(state.clusters[1].labels).toEqual([]);
    expect(state.currentCluster?.labels).toEqual([label]);
  });

  it('replaces the cluster snapshot after a detach', () => {
    const initial = stateWith({
      clusters: [buildCluster('c1', [label])],
      currentCluster: buildCluster('c1', [label]),
    });
    const refreshed = buildCluster('c1', []);

    const state = reducer(
      initial,
      detachKubernetesClusterLabel.fulfilled(refreshed, 'req', {
        clusterId: 'c1',
        labelId: 'l1',
      }),
    );

    expect(state.clusters[0].labels).toEqual([]);
    expect(state.currentCluster?.labels).toEqual([]);
  });

  it('does not touch the current cluster when another cluster changes', () => {
    const initial = stateWith({
      clusters: [buildCluster('c1'), buildCluster('c2')],
      currentCluster: buildCluster('c2'),
    });
    const refreshed = buildCluster('c1', [label]);

    const state = reducer(
      initial,
      attachKubernetesClusterLabel.fulfilled(refreshed, 'req', {
        clusterId: 'c1',
        labelId: 'l1',
      }),
    );

    expect(state.currentCluster?.labels).toEqual([]);
  });
});
