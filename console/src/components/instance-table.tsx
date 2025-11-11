import { ButtonGroup, IconButton } from '@chakra-ui/react';
import {
  Hypervisor,
  Instance,
  InstanceStatus,
  Organization,
  Project,
  ZeroTrustNetwork,
  Zone,
} from '@france-nuage/sdk';
import {
  CellContext,
  ColumnDef,
  Row,
  createColumnHelper,
} from '@tanstack/react-table';
import { FunctionComponent, ReactNode, useMemo } from 'react';
import { HiTrash } from 'react-icons/hi';
import { HiPlay, HiStop } from 'react-icons/hi2';

import { removeInstance, startInstance, stopInstance } from '@/features';
import { useAppDispatch } from '@/hooks';
import { bytesToGB } from '@/services';

import { AppTable } from './app-table';

export type InstanceTableProps = {
  zones: Zone[];
  hypervisors: Hypervisor[];
  instances: Instance[];
  organizations: Organization[];
  projects: Project[];
  vpcs: ZeroTrustNetwork[];
};

type InstanceData = Instance & {
  zone?: Zone;
  hypervisor?: Hypervisor;
  vpc?: ZeroTrustNetwork;
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
  columnHelper.accessor('zone.name', {
    header: 'Zone',
    id: 'zoneName',
  }),
  columnHelper.accessor('vpc.name', { header: 'Vpc', id: 'vpcName' }),
  columnHelper.accessor('organization.name', {
    header: 'Organization',
    id: 'organizationName',
  }),
  columnHelper.accessor('project.name', {
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
  vpcs,
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
        const vpc = vpcs.find((vpc) => vpc.id === instance.zeroTrustNetworkId)!;

        return {
          ...instance,
          hypervisor,
          organization,
          project,
          vpc,
          zone,
        };
      }),
    [zones, hypervisors, instances, organizations, projects, vpcs],
  );

  return <AppTable columns={columns} data={data} />;
};

export const ActionsCell = ({ row }: { row: Row<InstanceData> }) => {
  type Action = 'start' | 'stop' | 'remove';

  const dispatch = useAppDispatch();

  const actions: Record<Action, ReactNode> = {
    remove: (
      <IconButton
        aria-label="remove instance"
        bg={{ _hover: 'bg.error', base: 'transparent' }}
        color="fg.error"
        disabled
        onClick={() => dispatch(removeInstance(row.original.id))}
      >
        <HiTrash />
      </IconButton>
    ),
    start: (
      <IconButton
        aria-label="start instance"
        onClick={() => dispatch(startInstance(row.original.id))}
      >
        <HiPlay />
      </IconButton>
    ),
    stop: (
      <IconButton
        aria-label="stop instance"
        onClick={() => dispatch(stopInstance(row.original.id))}
      >
        <HiStop />
      </IconButton>
    ),
  };

  const matrix: Record<InstanceStatus, Action[]> = {
    [InstanceStatus.Deprovisionning]: [],
    [InstanceStatus.Provisioning]: [],
    [InstanceStatus.Repairing]: [],
    [InstanceStatus.Running]: ['stop', 'remove'],
    [InstanceStatus.Staging]: [],
    [InstanceStatus.Stopped]: ['start', 'remove'],
    [InstanceStatus.Stopping]: [],
    [InstanceStatus.Suspended]: [],
    [InstanceStatus.Suspending]: [],
    [InstanceStatus.Terminated]: [],
    [InstanceStatus.UndefinedInstanceStatus]: [],
  };

  return (
    <ButtonGroup size="xs" variant="ghost">
      {...matrix[row.original.status].map((status) => actions[status])}
    </ButtonGroup>
  );
};
