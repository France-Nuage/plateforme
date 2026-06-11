import { KubernetesLabel } from '@france-nuage/sdk';
import { useCallback, useEffect, useState } from 'react';

import { createKubernetesLabel, fetchKubernetesLabels } from '@/features';
import { getErrorMessage } from '@/utils';

import { useAppDispatch } from './use-app-dispatch';
import { useAppSelector } from './use-app-selector';

/**
 * Access to the platform-wide label registry.
 *
 * Loads the registry on mount and exposes the creation of new labels.
 * Creating a label is independent from any cluster: the label exists in the
 * registry immediately and can then be attached wherever needed.
 */
export function useKubernetesLabelRegistry() {
  const dispatch = useAppDispatch();
  const registryLabels = useAppSelector(
    (state) => state.kubernetesClusters.labels,
  );
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    dispatch(fetchKubernetesLabels());
  }, [dispatch]);

  /**
   * Creates a label in the registry. Resolves with the created label, or
   * `null` when the creation failed (the error is exposed via `error`).
   */
  const createLabel = useCallback(
    (key: string, value: string): Promise<KubernetesLabel | null> => {
      setBusy(true);
      setError(null);
      return dispatch(createKubernetesLabel({ key, value }))
        .unwrap()
        .then((label) => {
          setBusy(false);
          return label;
        })
        .catch((err: unknown) => {
          setError(getErrorMessage(err));
          setBusy(false);
          return null;
        });
    },
    [dispatch],
  );

  return {
    busy,
    createLabel,
    error,
    registryLabels: registryLabels ?? [],
  };
}
