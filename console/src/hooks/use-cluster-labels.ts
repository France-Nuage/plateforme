import { KubernetesLabel, ManagedServiceRef } from '@france-nuage/sdk';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

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

const EMPTY_LABELS: KubernetesLabel[] = [];

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

  const attachedLabels =
    currentCluster?.id === clusterId
      ? (currentCluster.labels ?? EMPTY_LABELS)
      : EMPTY_LABELS;
  const availableLabels = useMemo(
    () =>
      (registryLabels ?? EMPTY_LABELS).filter(
        (label) =>
          !label.system &&
          !attachedLabels.some((attached) => attached.id === label.id),
      ),
    [registryLabels, attachedLabels],
  );

  const runAction = useCallback(
    (promise: Promise<unknown>): Promise<boolean> => {
      setBusy(true);
      setError(null);
      return promise
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
    [],
  );

  /** Attaches an existing registry label to the cluster. */
  const attachExistingLabel = useCallback(
    (labelId: string): Promise<boolean> =>
      runAction(
        dispatch(attachKubernetesClusterLabel({ clusterId, labelId })).unwrap(),
      ),
    [clusterId, dispatch, runAction],
  );

  /** Creates a new registry label, then attaches it to the cluster. */
  const createAndAttachLabel = useCallback(
    (key: string, value: string): Promise<boolean> =>
      runAction(
        dispatch(createKubernetesLabel({ key, value }))
          .unwrap()
          .then((label) =>
            dispatch(
              attachKubernetesClusterLabel({ clusterId, labelId: label.id }),
            ).unwrap(),
          ),
      ),
    [clusterId, dispatch, runAction],
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

  const detachRef = useRef(detachConfirmation);
  detachRef.current = detachConfirmation;

  /** Detaches the label the dialog is currently confirming. */
  const confirmDetach = useCallback(() => {
    const current = detachRef.current;
    if (current.status !== 'ready') {
      return;
    }
    const { label, services } = current;
    setDetachConfirmation({ label, services, status: 'detaching' });
    dispatch(detachKubernetesClusterLabel({ clusterId, labelId: label.id }))
      .unwrap()
      .then(() => setDetachConfirmation({ status: 'idle' }))
      .catch((err: unknown) => {
        setError(getErrorMessage(err));
        setDetachConfirmation({ status: 'idle' });
      });
  }, [clusterId, dispatch]);

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
