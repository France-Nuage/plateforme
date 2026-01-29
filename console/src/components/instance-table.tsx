import {
  Button,
  ButtonGroup,
  Dialog,
  IconButton,
  Portal,
  Select,
  useListCollection,
} from '@chakra-ui/react';
import {
  Hypervisor,
  Instance,
  InstanceStatus,
  Organization,
  Project,
  Zone,
} from '@france-nuage/sdk';
import {
  CellContext,
  ColumnDef,
  Row,
  createColumnHelper,
} from '@tanstack/react-table';
import { FunctionComponent, ReactNode, useMemo, useState } from 'react';
import { HiTrash } from 'react-icons/hi';
import { HiArrowRight, HiPlay, HiStop } from 'react-icons/hi2';

import {
  removeInstance,
  startInstance,
  stopInstance,
  updateInstance,
} from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';
import { bytesToGB } from '@/services';

import { AppTable } from './app-table';

export type InstanceTableProps = {
  zones: Zone[];
  hypervisors: Hypervisor[];
  instances: Instance[];
  organizations: Organization[];
  projects: Project[];
};

type InstanceData = Instance & {
  zone?: Zone;
  hypervisor?: Hypervisor;
  organization?: Organization;
  project?: Project;
};

const columnHelper = createColumnHelper<InstanceData>();

const displayBytesColumn = (cell: CellContext<InstanceData, number>) =>
  bytesToGB(cell.getValue());

const date = (cell: CellContext<InstanceData, string>) =>
  new Date(cell.getValue()).toLocaleString();

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const columns: ColumnDef<InstanceData, any>[] = [
  columnHelper.accessor('name', {
    enableHiding: false,
    header: 'Name',
    id: 'name',
  }),
  columnHelper.accessor('id', {}),
  columnHelper.accessor((row) => row.zone?.name ?? '', {
    header: 'Zone',
    id: 'zoneName',
  }),
  columnHelper.accessor((row) => row.organization?.name ?? '', {
    header: 'Organization',
    id: 'organizationName',
  }),
  columnHelper.accessor((row) => row.project?.name ?? '', {
    header: 'Project',
    id: 'projectName',
  }),
  columnHelper.accessor('ipV4', { header: 'IpV4', id: 'ipv4' }),
  columnHelper.accessor('status', { header: 'Status', id: 'status' }),
  columnHelper.accessor('maxCpuCores', {
    cell: (cell) => `${cell.getValue()} coeur${cell.getValue() > 1 ? 's' : ''}`,
    header: 'Cpu Max',
    id: 'maxCpuCores',
  }),
  columnHelper.accessor('cpuUsagePercent', {
    cell: (cell) => `${Math.round(cell.getValue() * 100)}%`,
    header: 'Cpu Usage',
    id: 'cpuUsagePercent',
  }),
  columnHelper.accessor('maxMemoryBytes', {
    cell: displayBytesColumn,
    header: 'Ram Max',
    id: 'maxMemoryBytes',
  }),
  columnHelper.accessor('memoryUsageBytes', {
    cell: displayBytesColumn,
    header: 'Ram Usage',
    id: 'memoryUsageBytes',
  }),
  columnHelper.accessor('maxDiskBytes', {
    cell: displayBytesColumn,
    header: 'Disk Max',
    id: 'maxDiskBytes',
  }),
  columnHelper.accessor('diskUsageBytes', {
    cell: displayBytesColumn,
    header: 'Disk Usage',
    id: 'diskUsageBytes',
  }),
  columnHelper.accessor('createdAt', {
    cell: date,
    header: 'Created At',
    id: 'createdAt',
  }),
  columnHelper.accessor('updatedAt', {
    cell: date,
    header: 'Updated At',
    id: 'updatedAt',
  }),
  columnHelper.display({
    cell: ({ row }) => <ActionsCell row={row} />,
    header: 'Actions',
    id: 'actions',
  }),
];

export const InstanceTable: FunctionComponent<InstanceTableProps> = ({
  hypervisors,
  instances,
  organizations,
  projects,
  zones,
}) => {
  // Compute the instances data with associated relations.
  const data: InstanceData[] = useMemo(
    () =>
      instances.map((instance) => {
        const hypervisor = hypervisors.find(
          (hypervisor) => hypervisor.id === instance.hypervisorId,
        );
        const zone = zones.find((zone) => zone.id === hypervisor?.zoneId);
        const project = projects.find(
          (project) => project.id === instance.projectId,
        );
        const organization = organizations.find(
          (organization) => organization.id === project?.organizationId,
        );

        return {
          ...instance,
          hypervisor,
          organization,
          project,
          zone,
        };
      }),
    [zones, hypervisors, instances, organizations, projects],
  );

  return (
    <AppTable
      columns={columns}
      data={data}
      bulkActions={(selectedInstances) => (
        <InstanceBulkActions instances={selectedInstances} />
      )}
    />
  );
};

export const ActionsCell = ({ row }: { row: Row<InstanceData> }) => {
  type Action = 'start' | 'stop' | 'remove' | 'move';

  const dispatch = useAppDispatch();

  const [confirmation, setConfirmation] = useState<
    | {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        action: () => Promise<any>;
        description: string;
        title: string;
      }
    | undefined
  >(undefined);

  const [actionPending, setActionPending] = useState(false);

  const actions: Record<Action, ReactNode> = {
    move: <MoveToProjectButton instance={row.original} />,
    remove: (
      <Dialog.Root
        lazyMount
        unmountOnExit={false}
        open={!!confirmation}
        onOpenChange={(e) => !e.open && setConfirmation(undefined)}
      >
        <Dialog.Trigger asChild>
          <IconButton
            aria-label="remove instance"
            bg={{ _hover: 'bg.error', base: 'transparent' }}
            color="fg.error"
            onClick={() =>
              setConfirmation({
                action: () => dispatch(removeInstance(row.original.id)),
                description: `Êtes vous sûr de vouloir supprimer l'instance "${row.original.name}"`,
                title: "Supprimer l'instance",
              })
            }
          >
            <HiTrash />
          </IconButton>
        </Dialog.Trigger>
        <Portal>
          <Dialog.Backdrop />
          <Dialog.Positioner>
            <Dialog.Content>
              <Dialog.CloseTrigger />
              <Dialog.Header>
                <Dialog.Title>{confirmation?.title}</Dialog.Title>
              </Dialog.Header>
              <Dialog.Body>{confirmation?.description}</Dialog.Body>
              <Dialog.Footer>
                <Dialog.ActionTrigger asChild>
                  <Button disabled={actionPending} variant="outline">
                    Annuler
                  </Button>
                </Dialog.ActionTrigger>
                <Button
                  colorPalette="red"
                  disabled={actionPending}
                  loading={actionPending}
                  loadingText="Suppression en cours..."
                  onClick={async () => {
                    setActionPending(true);
                    await confirmation?.action();
                    setActionPending(false);
                    setConfirmation(undefined);
                  }}
                  variant="solid"
                >
                  Supprimer
                </Button>
              </Dialog.Footer>
            </Dialog.Content>
          </Dialog.Positioner>
        </Portal>
      </Dialog.Root>
    ),
    start: <StartInstanceButton instance={row.original} />,
    stop: <StopInstanceButton instance={row.original} />,
  };

  const matrix: Record<InstanceStatus, Action[]> = {
    [InstanceStatus.Deprovisionning]: [],
    [InstanceStatus.Provisioning]: [],
    [InstanceStatus.Repairing]: [],
    [InstanceStatus.Running]: ['move', 'stop', 'remove'],
    [InstanceStatus.Staging]: [],
    [InstanceStatus.Stopped]: ['move', 'start', 'remove'],
    [InstanceStatus.Stopping]: [],
    [InstanceStatus.Suspended]: [],
    [InstanceStatus.Suspending]: [],
    [InstanceStatus.Terminated]: [],
    [InstanceStatus.UndefinedInstanceStatus]: [],
  };

  return (
    <>
      <ButtonGroup size="xs" variant="ghost">
        {matrix[row.original.status].map((status) => actions[status])}
      </ButtonGroup>
    </>
  );
};

/**
 * Provides & button for starting the instance
 */
const StartInstanceButton: FunctionComponent<{ instance: Instance }> = ({
  instance,
}) => {
  const dispatch = useAppDispatch();
  const [loading, setLoading] = useState(false);

  const handleClick = () => {
    setLoading(true);
    dispatch(startInstance(instance.id))
      .catch((error) => console.error(error))
      .finally(() => setLoading(false));
  };

  return (
    <IconButton
      aria-label="start instance"
      onClick={handleClick}
      loading={loading}
    >
      <HiPlay />
    </IconButton>
  );
};

/**
 * Provides a button for stopping the instance
 */
const StopInstanceButton: FunctionComponent<{ instance: Instance }> = ({
  instance,
}) => {
  const dispatch = useAppDispatch();
  const [loading, setLoading] = useState(false);

  const handleClick = () => {
    setLoading(true);
    dispatch(stopInstance(instance.id))
      .catch((error) => console.error(error))
      .finally(() => setLoading(false));
  };

  return (
    <IconButton
      aria-label="stop instance"
      onClick={handleClick}
      loading={loading}
    >
      <HiStop />
    </IconButton>
  );
};

/**
 * Provides a button for moving the instance to a different project
 */
const MoveToProjectButton: FunctionComponent<{ instance: Instance }> = ({
  instance,
}) => {
  const dispatch = useAppDispatch();
  const projects = useAppSelector((state) => state.resources.projects);
  const organizations = useAppSelector(
    (state) => state.resources.organizations,
  );
  const [dialogOpen, setDialogOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [selectedProjectId, setSelectedProjectId] = useState<string[]>([]);

  // Filter out the current project from the list and build items with org/project labels
  const projectItems = useMemo(
    () =>
      projects
        .filter((p) => p.id !== instance.projectId)
        .map((project) => {
          const organization = organizations.find(
            (org) => org.id === project.organizationId,
          );
          return {
            label: `${organization?.name ?? 'Unknown'} / ${project.name}`,
            value: project.id,
          };
        }),
    [projects, instance.projectId, organizations],
  );

  const { collection } = useListCollection({
    initialItems: projectItems,
  });

  const handleMove = async () => {
    if (selectedProjectId.length === 0) return;

    setLoading(true);
    try {
      await dispatch(
        updateInstance({
          data: {
            image: '',
            maxCpuCores: instance.maxCpuCores,
            maxDiskBytes: instance.maxDiskBytes,
            maxMemoryBytes: instance.maxMemoryBytes,
            name: instance.name,
            projectId: selectedProjectId[0],
            snippet: '',
          },
          id: instance.id,
        }),
      );
      setDialogOpen(false);
      setSelectedProjectId([]);
    } catch (error) {
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog.Root
      lazyMount
      open={dialogOpen}
      onOpenChange={(e) => {
        setDialogOpen(e.open);
        if (!e.open) {
          setSelectedProjectId([]);
        }
      }}
    >
      <Dialog.Trigger asChild>
        <IconButton aria-label="move to project">
          <HiArrowRight />
        </IconButton>
      </Dialog.Trigger>
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content>
            <Dialog.CloseTrigger />
            <Dialog.Header>
              <Dialog.Title>Déplacer vers un projet</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <p style={{ marginBottom: '1rem' }}>
                Sélectionnez le projet de destination pour l&apos;instance
                &quot;
                {instance.name}&quot;
              </p>
              <Select.Root
                collection={collection}
                value={selectedProjectId}
                onValueChange={(e) => setSelectedProjectId(e.value)}
              >
                <Select.Control>
                  <Select.Trigger>
                    <Select.ValueText placeholder="Sélectionner un projet" />
                  </Select.Trigger>
                  <Select.IndicatorGroup>
                    <Select.Indicator />
                  </Select.IndicatorGroup>
                </Select.Control>
                <Select.Positioner>
                  <Select.Content>
                    {projectItems.map((item) => (
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
                disabled={loading || selectedProjectId.length === 0}
                loading={loading}
                loadingText="Déplacement en cours..."
                onClick={handleMove}
                variant="solid"
              >
                Déplacer
              </Button>
            </Dialog.Footer>
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  );
};

/**
 * Bulk actions component for selected instances.
 *
 * Displays action buttons with counts based on instance states:
 * - Start: available for stopped instances
 * - Stop: available for running instances
 * - Delete: available for stopped instances (with confirmation dialog)
 */
const InstanceBulkActions: FunctionComponent<{ instances: InstanceData[] }> = ({
  instances,
}) => {
  const dispatch = useAppDispatch();
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [deleteLoading, setDeleteLoading] = useState(false);

  const startableInstances = instances.filter(
    (instance) => instance.status === InstanceStatus.Stopped,
  );
  const stoppableInstances = instances.filter(
    (instance) => instance.status === InstanceStatus.Running,
  );
  const deletableInstances = instances.filter(
    (instance) => instance.status === InstanceStatus.Stopped,
  );

  const handleStart = () => {
    startableInstances.forEach((instance) =>
      dispatch(startInstance(instance.id)),
    );
  };

  const handleStop = () => {
    stoppableInstances.forEach((instance) =>
      dispatch(stopInstance(instance.id)),
    );
  };

  const handleDelete = async () => {
    setDeleteLoading(true);
    await Promise.all(
      deletableInstances.map((instance) =>
        dispatch(removeInstance(instance.id)),
      ),
    );
    setDeleteLoading(false);
    setDeleteDialogOpen(false);
  };

  return (
    <>
      <Button
        variant="outline"
        size="sm"
        onClick={handleStart}
        disabled={startableInstances.length === 0}
      >
        <HiPlay />
        Start ({startableInstances.length})
      </Button>
      <Button
        variant="outline"
        size="sm"
        onClick={handleStop}
        disabled={stoppableInstances.length === 0}
      >
        <HiStop />
        Stop ({stoppableInstances.length})
      </Button>
      <Dialog.Root
        lazyMount
        open={deleteDialogOpen}
        onOpenChange={(e) => setDeleteDialogOpen(e.open)}
      >
        <Dialog.Trigger asChild>
          <Button
            variant="outline"
            size="sm"
            disabled={deletableInstances.length === 0}
            colorPalette="red"
          >
            <HiTrash />
            Delete ({deletableInstances.length})
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
                Êtes-vous sûr de vouloir supprimer {deletableInstances.length}{' '}
                instance{deletableInstances.length > 1 ? 's' : ''} ?
              </Dialog.Body>
              <Dialog.Footer>
                <Dialog.ActionTrigger asChild>
                  <Button disabled={deleteLoading} variant="outline">
                    Annuler
                  </Button>
                </Dialog.ActionTrigger>
                <Button
                  colorPalette="red"
                  disabled={deleteLoading}
                  loading={deleteLoading}
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
    </>
  );
};
