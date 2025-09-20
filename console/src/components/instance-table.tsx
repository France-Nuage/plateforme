import {
  CellContext,
  ColumnDef,
  createColumnHelper,
} from '@tanstack/react-table';
import { FunctionComponent, useMemo } from 'react';

import { Organization, Project } from '@/generated/rpc/resources';
import { Datacenter, Hypervisor, Instance, ZeroTrustNetwork } from '@/types';

import { AppTable } from './app-table';

export type InstanceTableProps = {
  datacenters: Datacenter[];
  hypervisors: Hypervisor[];
  instances: Instance[];
  organizations: Organization[];
  projects: Project[];
  vpcs: ZeroTrustNetwork[];
};

type InstanceData = Instance & {
  datacenter?: Datacenter;
  hypervisor?: Hypervisor;
  vpc?: ZeroTrustNetwork;
  organization?: Organization;
  project?: Project;
};

const columnHelper = createColumnHelper<InstanceData>();

const bytesToGB = (cell: CellContext<InstanceData, number>) =>
  `${(cell.getValue() / 1024 ** 3).toFixed(2)}GB`;
const date = (cell: CellContext<InstanceData, string>) =>
  new Date(cell.getValue()).toLocaleString();

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const columns: ColumnDef<InstanceData, any>[] = [
  columnHelper.accessor('name', {
    enableHiding: false,
    header: 'Name',
    id: 'name',
  }),
  columnHelper.accessor('datacenter.name', {
    header: 'Datacenter',
    id: 'datacenterName',
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
    cell: bytesToGB,
    header: 'Ram Max',
    id: 'maxMemoryBytes',
  }),
  columnHelper.accessor('memoryUsageBytes', {
    cell: bytesToGB,
    header: 'Ram Usage',
    id: 'memoryUsageBytes',
  }),
  columnHelper.accessor('maxDiskBytes', {
    cell: bytesToGB,
    header: 'Disk Max',
    id: 'maxDiskBytes',
  }),
  columnHelper.accessor('diskUsageBytes', {
    cell: bytesToGB,
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
];

export const InstanceTable: FunctionComponent<InstanceTableProps> = ({
  datacenters,
  hypervisors,
  instances,
  organizations,
  projects,
  vpcs,
}) => {
  // Compute the instances data with associated relations.
  const data: InstanceData[] = useMemo(
    () =>
      instances.map((instance) => {
        const hypervisor = hypervisors.find(
          (hypervisor) => hypervisor.id === instance.hypervisorId,
        );
        const datacenter = datacenters.find(
          (datacenter) => datacenter.id === hypervisor?.datacenterId,
        );
        const project = projects.find(
          (project) => project.id === instance.projectId,
        );
        const organization = organizations.find(
          (organization) => organization.id === project?.organizationId,
        );
        const vpc = vpcs.find((vpc) => vpc.id === instance.zeroTrustNetworkId)!;

        return {
          ...instance,
          datacenter,
          hypervisor,
          organization,
          project,
          vpc,
        };
      }),
    [datacenters, hypervisors, instances, organizations, projects, vpcs],
  );

  return <AppTable columns={columns} data={data} />;
};
