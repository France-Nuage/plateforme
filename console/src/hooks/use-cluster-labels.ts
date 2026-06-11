import { KubernetesLabel, ManagedServiceRef } from '@france-nuage/sdk';
import { useCallback, useEffect, useState } from 'react';

import {
  attachKubernetesClusterLabel,
  createKubernetesLabel,
  detachKubernetesClusterLabel,
  fetchKubernetesLabels,
  fetchServicesReferencingLabel,
} from '@/features';
import { getErrorMessage } from '@/utils';

import { useAppDispatch } from './use-app-dispatch';
import { useAppSelector } from './use-app-selector';

/**
 * State machine of the detach confirmation dialog.
 *
 * Detaching a label is always allowed (placement only happens at deployment
 * time), but the dialog first loads the managed services whose deploy_target
 * requires the label so the operator confirms with full knowledge of the
 * impact on future deployments.
 */
export type DetachConfirmation =
  | { status: 'idle' }
  | { status: 'loading'; label: KubernetesLabel }
  | { status: 'ready'; label: KubernetesLabel; services: ManagedServiceRef[] }
  | {
      status: 'detaching';
      label: KubernetesLabel;
      services: ManagedServiceRef[];
    };

/**
 * Business logic of the cluster label editor.
 *
 * Loads the platform label registry, derives the labels attachable to the
 * cluster (non-system, not already attached), and exposes the attach,
 * create-and-attach, and guarded detach flows. The presentation component
 * consumes this hook and renders, nothing else.
 */
export function useClusterLabels(clusterId: string) {
  const dispatch = useAppDispatch();
  const currentCluster = useAppSelector(
    (state) => state.kubernetesClusters.currentCluster,
  );
  const registryLabels = useAppSelector(
    (state) => state.kubernetesClusters.labels,
  );

  const [detachConfirmation, setDetachConfirmation] =
    useState<DetachConfirmation>({ status: 'idle' });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    dispatch(fetchKubernetesLabels());
  }, [dispatch]);

  // The `?? []` fallbacks guard against cluster snapshots predating the
  // labels feature (stale store state across a hot reload, older backend):
  // an empty list degrades gracefully where `undefined.map` blanks the page.
  const attachedLabels =
    (currentCluster?.id === clusterId ? currentCluster.labels : []) ?? [];
  const availableLabels = (registryLabels ?? []).filter(
    (label) =>
      !label.system &&
      !attachedLabels.some((attached) => attached.id === label.id),
  );

  /** Attaches an existing registry label to the cluster. */
  const attachExistingLabel = useCallback(
    (labelId: string): Promise<boolean> => {
      setBusy(true);
      setError(null);
      return dispatch(attachKubernetesClusterLabel({ clusterId, labelId }))
        .unwrap()
        .then(() => {
          setBusy(false);
          return true;
        })
        .catch((err: unknown) => {
          setError(getErrorMessage(err));
          setBusy(false);
          return false;
        });
    },
    [clusterId, dispatch],
  );

  /** Creates a new registry label, then attaches it to the cluster. */
  const createAndAttachLabel = useCallback(
    (key: string, value: string): Promise<boolean> => {
      setBusy(true);
      setError(null);
      return dispatch(createKubernetesLabel({ key, value }))
        .unwrap()
        .then((label) =>
          dispatch(
            attachKubernetesClusterLabel({ clusterId, labelId: label.id }),
          ).unwrap(),
        )
        .then(() => {
          setBusy(false);
          return true;
        })
        .catch((err: unknown) => {
          setError(getErrorMessage(err));
          setBusy(false);
          return false;
        });
    },
    [clusterId, dispatch],
  );

  /**
   * Opens the detach confirmation and loads the impacted managed services.
   */
  const requestDetach = useCallback(
    (label: KubernetesLabel) => {
      setError(null);
      setDetachConfirmation({ label, status: 'loading' });
      dispatch(fetchServicesReferencingLabel(label.id))
        .unwrap()
        .then((services) =>
          setDetachConfirmation((current) =>
            current.status === 'loading' && current.label.id === label.id
              ? { label, services, status: 'ready' }
              : current,
          ),
        )
        .catch((err: unknown) => {
          setError(getErrorMessage(err));
          setDetachConfirmation({ status: 'idle' });
        });
    },
    [dispatch],
  );

  /** Closes the confirmation dialog without detaching. */
  const cancelDetach = useCallback(
    () => setDetachConfirmation({ status: 'idle' }),
    [],
  );

  /** Detaches the label the dialog is currently confirming. */
  const confirmDetach = useCallback(() => {
    if (detachConfirmation.status !== 'ready') {
      return;
    }
    const { label, services } = detachConfirmation;
    setDetachConfirmation({ label, services, status: 'detaching' });
    dispatch(detachKubernetesClusterLabel({ clusterId, labelId: label.id }))
      .unwrap()
      .then(() => setDetachConfirmation({ status: 'idle' }))
      .catch((err: unknown) => {
        setError(getErrorMessage(err));
        setDetachConfirmation({ status: 'idle' });
      });
  }, [detachConfirmation, clusterId, dispatch]);

  return {
    attachedLabels,
    attachExistingLabel,
    availableLabels,
    busy,
    cancelDetach,
    confirmDetach,
    createAndAttachLabel,
    detachConfirmation,
    error,
    requestDetach,
  };
}
