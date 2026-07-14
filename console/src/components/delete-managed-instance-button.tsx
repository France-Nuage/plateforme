import { ManagedServiceInstance } from '@france-nuage/sdk';
import { FunctionComponent } from 'react';

import { DeleteEntityButton } from '@/components/delete-entity-button';
import { deleteManagedInstance } from '@/features';
import { useAppDispatch } from '@/hooks';

export type DeleteManagedInstanceButtonProps = {
  instance: ManagedServiceInstance;
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
 * Confirmation dialog wrapping the delete-instance dispatch.
 *
 * Used by the listing table (icon-only flavor) and the instance detail
 * page (labelled flavor with navigation after deletion).
 */
export const DeleteManagedInstanceButton: FunctionComponent<
  DeleteManagedInstanceButtonProps
> = ({ instance, onDeleted, label, variant = 'outline', size = 'sm' }) => {
  const dispatch = useAppDispatch();

  return (
    <DeleteEntityButton
      entityName={instance.releaseName}
      onConfirm={() => dispatch(deleteManagedInstance(instance.id))}
      onDeleted={onDeleted}
      label={label}
      variant={variant}
      size={size}
      dialogTitle="Supprimer l'instance"
      confirmationPrefix="Confirmer la suppression de l'instance"
      iconAriaLabel="supprimer l'instance"
    />
  );
};
