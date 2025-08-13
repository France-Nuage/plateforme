import { Span, Stack, Table } from '@chakra-ui/react';
import {
  SortingState,
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { FunctionComponent, useMemo, useState } from 'react';
import { FaSort, FaSortDown, FaSortUp } from 'react-icons/fa';

import { Organization, Project } from '@/generated/rpc/resources';
import { Datacenter, Hypervisor, Instance, ZeroTrustNetwork } from '@/types';

export type InstanceTableProps = {
  datacenters: Datacenter[];
  hypervisors: Hypervisor[];
  instances: Instance[];
  organizations: Organization[];
  projects: Project[];
  vpcs: ZeroTrustNetwork[];
};

type InstanceData = Instance & {
  datacenter: Datacenter;
  hypervisor: Hypervisor;
  vpc: ZeroTrustNetwork;
  organization: Organization;
  project: Project;
};

const columnHelper = createColumnHelper<InstanceData>();

const columns = [
  columnHelper.accessor('name', {}),
  columnHelper.accessor('maxCpuCores', {}),
  columnHelper.accessor('cpuUsagePercent', {}),
  columnHelper.accessor('maxMemoryBytes', {}),
  columnHelper.accessor('memoryUsageBytes', {}),
  columnHelper.accessor('maxDiskBytes', {}),
  columnHelper.accessor('diskUsageBytes', {}),
];

export const InstanceTable: FunctionComponent<InstanceTableProps> = ({
  datacenters,
  hypervisors,
  instances,
  organizations,
  projects,
  vpcs,
}) => {
  const data: InstanceData[] = useMemo(
    () =>
      instances.map((instance) => {
        const hypervisor = hypervisors.find(
          (hypervisor) => hypervisor.id === instance.hypervisorId,
        )!;
        const datacenter = datacenters.find(
          (datacenter) => datacenter.id === hypervisor?.id,
        )!;
        const project = projects.find(
          (project) => project.id === instance.projectId,
        )!;
        const organization = organizations.find(
          (organization) => organization.id === project?.organizationId,
        )!;
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

  const [sorting, setSorting] = useState<SortingState>([]);

  const table = useReactTable({
    columns,
    data,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onSortingChange: setSorting,
    state: { sorting },
  });

  return (
    <Stack>
      <Table.ScrollArea borderWidth={1}>
        <Table.Root>
          <Table.Header>
            {table.getHeaderGroups().map((headerGroup) => (
              <Table.Row key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <Table.ColumnHeader
                    key={header.id}
                    onClick={header.column.getToggleSortingHandler()}
                    whiteSpace="nowrap"
                  >
                    {flexRender(
                      header.column.columnDef.header,
                      header.getContext(),
                    )}
                    <Span ml={1} css={{ '& svg': { display: 'inline' } }}>
                      {header.column.getIsSorted() ? (
                        header.column.getIsSorted() === 'desc' ? (
                          <FaSortDown />
                        ) : (
                          <FaSortUp />
                        )
                      ) : (
                        <FaSort />
                      )}
                    </Span>
                  </Table.ColumnHeader>
                ))}
              </Table.Row>
            ))}
          </Table.Header>
          <Table.Body>
            {table.getRowModel().rows.map((row) => (
              <Table.Row key={row.id}>
                {row.getVisibleCells().map((cell) => (
                  <Table.Cell key={cell.id}>
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </Table.Cell>
                ))}
              </Table.Row>
            ))}
          </Table.Body>
        </Table.Root>
      </Table.ScrollArea>
    </Stack>
  );
};
