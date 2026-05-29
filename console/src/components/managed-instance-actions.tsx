import {
  Button,
  Dialog,
  IconButton,
  Portal,
  Select,
  Text,
  useListCollection,
} from '@chakra-ui/react';
import {
  ManagedServiceInstance,
  ManagedServiceVersion,
} from '@france-nuage/sdk';
import { FunctionComponent, useMemo, useState } from 'react';
import { HiTrash } from 'react-icons/hi';

import { deleteManagedInstance, upgradeManagedInstance } from '@/features';
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
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);

  const handleDelete = () => {
    setLoading(true);
    dispatch(deleteManagedInstance(instance.id))
      .then(() => {
        setOpen(false);
        setLoading(false);
        onDeleted?.();
      })
      .catch(() => setLoading(false));
  };

  return (
    <Dialog.Root
      lazyMount
      unmountOnExit={false}
      open={open}
      onOpenChange={(event) => setOpen(event.open)}
    >
      <Dialog.Trigger asChild>
        {label ? (
          <Button colorPalette="red" variant={variant} size={size}>
            <HiTrash />
            {label}
          </Button>
        ) : (
          <IconButton
            aria-label="supprimer l'instance"
            bg={{ _hover: 'bg.error', base: 'transparent' }}
            color="fg.error"
          >
            <HiTrash />
          </IconButton>
        )}
      </Dialog.Trigger>
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content>
            <Dialog.CloseTrigger />
            <Dialog.Header>
              <Dialog.Title>Supprimer l'instance</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              Confirmer la suppression de l'instance{' '}
              <strong>{instance.releaseName}</strong> ? Cette action est
              irréversible.
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

export type UpgradeManagedInstanceButtonProps = {
  instance: ManagedServiceInstance;
  versions: ManagedServiceVersion[];
};

/**
 * Dialog letting the user upgrade an instance to a newer chart version.
 *
 * Renders nothing when no upgrade target is available (every version is
 * either the current one or older).
 */
export const UpgradeManagedInstanceButton: FunctionComponent<
  UpgradeManagedInstanceButtonProps
> = ({ instance, versions }) => {
  const dispatch = useAppDispatch();
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [selectedVersionId, setSelectedVersionId] = useState<string[]>([]);

  const availableVersions = useMemo(
    () => versions.filter((version) => version.id !== instance.versionId),
    [versions, instance.versionId],
  );

  const versionItems = useMemo(
    () =>
      availableVersions.map((version) => ({
        label: `${version.chartVersion}${version.appVersion ? ` (app: ${version.appVersion})` : ''}`,
        value: version.id,
      })),
    [availableVersions],
  );

  const { collection } = useListCollection({ initialItems: versionItems });

  if (availableVersions.length === 0) return null;

  const handleUpgrade = () => {
    setLoading(true);
    dispatch(
      upgradeManagedInstance({
        instanceId: instance.id,
        versionId: selectedVersionId[0],
      }),
    )
      .then(() => {
        setOpen(false);
        setLoading(false);
        setSelectedVersionId([]);
      })
      .catch(() => setLoading(false));
  };

  return (
    <Dialog.Root
      lazyMount
      open={open}
      onOpenChange={(event) => {
        setOpen(event.open);
        if (!event.open) setSelectedVersionId([]);
      }}
    >
      <Dialog.Trigger asChild>
        <Button variant="outline" size="sm">
          Mettre à jour
        </Button>
      </Dialog.Trigger>
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content>
            <Dialog.CloseTrigger />
            <Dialog.Header>
              <Dialog.Title>Mise à jour de l'instance</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <Text mb={3}>Sélectionner la nouvelle version :</Text>
              <Select.Root
                collection={collection}
                value={selectedVersionId}
                onValueChange={(event) => setSelectedVersionId(event.value)}
              >
                <Select.Control>
                  <Select.Trigger>
                    <Select.ValueText placeholder="Version cible" />
                  </Select.Trigger>
                  <Select.IndicatorGroup>
                    <Select.Indicator />
                  </Select.IndicatorGroup>
                </Select.Control>
                <Select.Positioner>
                  <Select.Content>
                    {versionItems.map((item) => (
                      <Select.Item item={item} key={item.value}>
                        {item.label}
                        <Select.ItemIndicator />
                      </Select.Item>
                    ))}
                  </Select.Content>
                </Select.Positioner>
              </Select.Root>
            </Dialog.Body>
            <Dialog.Footer>
              <Dialog.ActionTrigger asChild>
                <Button disabled={loading} variant="outline">
                  Annuler
                </Button>
              </Dialog.ActionTrigger>
              <Button
                colorPalette="blue"
                disabled={loading || selectedVersionId.length === 0}
                loading={loading}
                loadingText="Mise à jour..."
                onClick={handleUpgrade}
                variant="solid"
              >
                Mettre à jour
              </Button>
            </Dialog.Footer>
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  );
};
