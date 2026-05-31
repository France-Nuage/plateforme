import { Button, Dialog, IconButton, Portal } from '@chakra-ui/react';
import { FunctionComponent, useState } from 'react';
import { HiTrash } from 'react-icons/hi';

export type DeleteEntityButtonProps = {
  /** Name of the entity shown in bold in the confirmation body. */
  entityName: string;
  /** Async action to execute when the user confirms deletion. */
  onConfirm: () => Promise<unknown>;
  /** Optional callback fired once the delete action resolves. */
  onDeleted?: () => void;
  /** When provided, renders a labelled button instead of the icon-only one. */
  label?: string;
  /** Button variant when `label` is set. */
  variant?: 'outline' | 'solid';
  /** Button size when `label` is set. */
  size?: 'xs' | 'sm' | 'md';
  /** Dialog title (e.g. "Supprimer le cluster"). */
  dialogTitle: string;
  /** Confirmation body prefix before the entity name (e.g. "Confirmer la suppression du cluster"). */
  confirmationPrefix: string;
  /** Aria label for the icon button variant. */
  iconAriaLabel: string;
};

/**
 * Generic confirmation dialog that wraps a delete action.
 *
 * Renders either an icon-only button or a labelled button depending on whether
 * `label` is provided. Manages its own open/loading state and calls `onDeleted`
 * once the action resolves.
 */
export const DeleteEntityButton: FunctionComponent<DeleteEntityButtonProps> = ({
  entityName,
  onConfirm,
  onDeleted,
  label,
  variant = 'outline',
  size = 'sm',
  dialogTitle,
  confirmationPrefix,
  iconAriaLabel,
}) => {
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);

  const handleDelete = () => {
    setLoading(true);
    onConfirm()
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
            aria-label={iconAriaLabel}
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
              <Dialog.Title>{dialogTitle}</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              {confirmationPrefix} <strong>{entityName}</strong> ? Cette action
              est irréversible.
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
