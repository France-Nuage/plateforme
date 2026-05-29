import {
  Button,
  ButtonGroup,
  Dialog,
  IconButton,
  Portal,
} from '@chakra-ui/react';
import {
  ManagedInstanceStatus,
  ManagedService,
  ManagedServiceInstance,
} from '@france-nuage/sdk';
import {
  CellContext,
  ColumnDef,
  Row,
  createColumnHelper,
} from '@tanstack/react-table';
import { FunctionComponent, ReactNode, useMemo, useState } from 'react';
import { HiInformationCircle, HiTrash } from 'react-icons/hi';
import { Link } from 'react-router';

import { deleteManagedInstance } from '@/features';
import { useAppDispatch } from '@/hooks';

import { AppTable } from './app-table';
import { DeleteManagedInstanceButton } from './managed-instance-actions';
import { ManagedInstanceStatusBadge } from './managed-instance-status-badge';

export type ManagedInstanceTableProps = {
  instances: ManagedServiceInstance[];
  services: ManagedService[];
};

type ManagedInstanceData = ManagedServiceInstance & {
  serviceName: string;
};

const columnHelper = createColumnHelper<ManagedInstanceData>();

const renderDate = (cell: CellContext<ManagedInstanceData, string>) =>
  new Date(cell.getValue()).toLocaleString();

const renderStatus = (
  cell: CellContext<ManagedInstanceData, ManagedInstanceStatus>,
) => <ManagedInstanceStatusBadge status={cell.getValue()} />;

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const columns: ColumnDef<ManagedInstanceData, any>[] = [
  columnHelper.accessor('serviceName', {
    enableHiding: false,
    header: 'Service',
    id: 'serviceName',
  }),
  columnHelper.accessor('releaseName', {
    header: 'Release',
    id: 'releaseName',
  }),
  columnHelper.accessor('namespace', {
    header: 'Namespace',
    id: 'namespace',
  }),
  columnHelper.accessor('status', {
    cell: renderStatus,
    header: 'Statut',
    id: 'status',
  }),
  columnHelper.accessor('createdAt', {
    cell: renderDate,
    header: 'Date de création',
    id: 'createdAt',
  }),
  columnHelper.accessor('id', { header: 'Id', id: 'id' }),
  columnHelper.display({
    cell: ({ row }) => <ActionsCell row={row} />,
    header: 'Actions',
    id: 'actions',
  }),
];

/**
 * Table listing managed service instances of the active project.
 *
 * Mirrors the architecture of `InstanceTable` (compute VMs): a thin wrapper
 * around `AppTable` (sort / search / column visibility / drag-reorder /
 * bulk-edit) with a declarative action matrix per status.
 */
export const ManagedInstanceTable: FunctionComponent<
  ManagedInstanceTableProps
> = ({ instances, services }) => {
  const data: ManagedInstanceData[] = useMemo(
    () =>
      instances.map((instance) => ({
        ...instance,
        serviceName:
          services.find((service) => service.id === instance.serviceId)?.name ??
          instance.serviceId,
      })),
    [instances, services],
  );

  return (
    <AppTable
      columns={columns}
      data={data}
      bulkActions={(selected) => (
        <ManagedInstanceBulkActions instances={selected} />
      )}
    />
  );
};

type Action = 'detail' | 'remove';

const ACTION_MATRIX: Record<ManagedInstanceStatus, Action[]> = {
  [ManagedInstanceStatus.Provisioning]: ['detail'],
  [ManagedInstanceStatus.Running]: ['detail', 'remove'],
  [ManagedInstanceStatus.Upgrading]: ['detail'],
  [ManagedInstanceStatus.Failed]: ['detail'],
  [ManagedInstanceStatus.Deleting]: ['detail'],
  [ManagedInstanceStatus.Deleted]: [],
};

const ActionsCell: FunctionComponent<{
  row: Row<ManagedInstanceData>;
}> = ({ row }) => {
  const actions: Record<Action, ReactNode> = {
    detail: <DetailInstanceButton instance={row.original} />,
    remove: <DeleteManagedInstanceButton instance={row.original} />,
  };

  return (
    <ButtonGroup size="xs" variant="ghost">
      {ACTION_MATRIX[row.original.status].map((action) => (
        <span key={action}>{actions[action]}</span>
      ))}
    </ButtonGroup>
  );
};

const DetailInstanceButton: FunctionComponent<{
  instance: ManagedServiceInstance;
}> = ({ instance }) => (
  <IconButton aria-label="voir le detail" asChild>
    <Link to={`/managed-services/instances/${instance.id}`}>
      <HiInformationCircle />
    </Link>
  </IconButton>
);

/**
 * Bulk action bar shown when at least one row is selected: delete every
 * deletable instance in the selection (Running or Failed).
 */
const ManagedInstanceBulkActions: FunctionComponent<{
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
