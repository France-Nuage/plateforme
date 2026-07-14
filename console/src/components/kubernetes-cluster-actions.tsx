import { KubernetesCluster } from '@france-nuage/sdk';
import { FunctionComponent } from 'react';

import { DeleteEntityButton } from '@/components/delete-entity-button';
import { deleteKubernetesCluster } from '@/features';
import { useAppDispatch } from '@/hooks';

export type DeleteKubernetesClusterButtonProps = {
  cluster: KubernetesCluster;
  /** Optional callback fired once the delete dispatch resolves. */
  onDeleted?: () => void;
  /** When provided, renders a labelled button instead of the icon-only one. */
  label?: string;
  /** Button variant when `label` is set. */
  variant?: 'outline' | 'solid';
  /** Button size when `label` is set. */
  size?: 'xs' | 'sm' | 'md';
};

/**
 * Confirmation dialog wrapping the delete-cluster dispatch.
 *
 * Used by the listing table (icon-only flavor) and the cluster detail page
 * (labelled flavor with navigation after deletion).
 */
export const DeleteKubernetesClusterButton: FunctionComponent<
  DeleteKubernetesClusterButtonProps
> = ({ cluster, onDeleted, label, variant = 'outline', size = 'sm' }) => {
  const dispatch = useAppDispatch();

  return (
    <DeleteEntityButton
      entityName={cluster.name}
      onConfirm={() => dispatch(deleteKubernetesCluster(cluster.id))}
      onDeleted={onDeleted}
      label={label}
      variant={variant}
      size={size}
      dialogTitle="Supprimer le cluster"
      confirmationPrefix="Confirmer la suppression du cluster"
      iconAriaLabel="supprimer le cluster"
    />
  );
};
