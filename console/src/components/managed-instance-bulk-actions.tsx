import { Button, Dialog, Portal } from '@chakra-ui/react';
import {
  ManagedInstanceStatus,
  ManagedServiceInstance,
} from '@france-nuage/sdk';
import { FunctionComponent, useMemo, useState } from 'react';
import { HiTrash } from 'react-icons/hi';

import { deleteManagedInstance } from '@/features';
import { useAppDispatch } from '@/hooks';

type ManagedInstanceData = ManagedServiceInstance & {
  serviceName: string;
};

/**
 * Bulk action bar shown when at least one row is selected: delete every
 * deletable instance in the selection (Running or Failed).
 */
export const ManagedInstanceBulkActions: FunctionComponent<{
  instances: ManagedInstanceData[];
}> = ({ instances }) => {
  const dispatch = useAppDispatch();
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);

  const deletables = useMemo(
    () =>
      instances.filter(
        (instance) => instance.status === ManagedInstanceStatus.Running,
      ),
    [instances],
  );

  const handleDelete = () => {
    setLoading(true);
    Promise.all(
      deletables.map((instance) =>
        dispatch(deleteManagedInstance(instance.id)),
      ),
    )
      .then(() => {
        setLoading(false);
        setOpen(false);
      })
      .catch(() => setLoading(false));
  };

  return (
    <Dialog.Root
      lazyMount
      open={open}
      onOpenChange={(event) => setOpen(event.open)}
    >
      <Dialog.Trigger asChild>
        <Button
          variant="outline"
          size="sm"
          disabled={deletables.length === 0}
          colorPalette="red"
        >
          <HiTrash />
          Supprimer ({deletables.length})
        </Button>
      </Dialog.Trigger>
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content>
            <Dialog.CloseTrigger />
            <Dialog.Header>
              <Dialog.Title>Supprimer les instances</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              Êtes-vous sûr de vouloir supprimer {deletables.length} instance
              {deletables.length > 1 ? 's' : ''} ?
            </Dialog.Body>
            <Dialog.Footer>
              <Dialog.ActionTrigger asChild>
                <Button disabled={loading} variant="outline">
                  Annuler
                </Button>
              </Dialog.ActionTrigger>
              <Button
                colorPalette="red"
                disabled={loading}
                loading={loading}
                loadingText="Suppression en cours..."
                onClick={handleDelete}
                variant="solid"
              >
                Supprimer
              </Button>
            </Dialog.Footer>
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  );
};
