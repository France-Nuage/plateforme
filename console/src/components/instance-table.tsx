import {
  ActionBar,
  Button,
  Checkbox,
  Portal,
  Span,
  Table,
  TableCell,
} from '@chakra-ui/react';
import {
  DndContext,
  DragEndEvent,
  KeyboardSensor,
  MouseSensor,
  TouchSensor,
  closestCenter,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import { restrictToHorizontalAxis } from '@dnd-kit/modifiers';
import {
  SortableContext,
  arrayMove,
  horizontalListSortingStrategy,
  useSortable,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import {
  Cell,
  CellContext,
  Header,
  SortingState,
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { CSSProperties, FunctionComponent, useMemo, useState } from 'react';
import { FaSort, FaSortDown, FaSortUp } from 'react-icons/fa';
import { PiDotsSixVertical } from 'react-icons/pi';

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

const bytesToGB = (cell: CellContext<InstanceData, number>) =>
  `${(cell.getValue() / 1024 ** 3).toFixed()} GB`;
const date = (cell: CellContext<InstanceData, string>) =>
  new Date(cell.getValue()).toLocaleString();

const columns = [
  columnHelper.accessor('name', { id: 'name' }),
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
    header: 'Cpu Max',
    id: 'maxCpuCores',
  }),
  columnHelper.accessor('cpuUsagePercent', {
    cell: (cell) => `${cell.getValue()} %`,
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

  // Track the column order.
  const [columnOrder, setColumnOrder] = useState<string[]>(() =>
    columns.map((column) => column.id!),
  );

  // Track the sort column.
  const [sorting, setSorting] = useState<SortingState>([]);

  // Create the react table
  const table = useReactTable({
    columns,
    data,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onColumnOrderChange: setColumnOrder,
    onSortingChange: setSorting,
    state: { columnOrder, sorting },
  });

  // Handle column reordering after drag & drop
  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (active && over && active.id !== over.id) {
      setColumnOrder((columnOrder) => {
        const oldIndex = columnOrder.indexOf(active.id as string);
        const newIndex = columnOrder.indexOf(over.id as string);
        return arrayMove(columnOrder, oldIndex, newIndex);
      });
    }
  };

  // Define the sensors for moving the columns
  const sensors = useSensors(
    useSensor(MouseSensor, {}),
    useSensor(TouchSensor, {}),
    useSensor(KeyboardSensor, {}),
  );

  // Track the selected rows
  const [selection, setSelection] = useState<string[]>([]);
  const indeterminate =
    selection.length > 0 && selection.length < instances.length;

  return (
    <DndContext
      collisionDetection={closestCenter}
      modifiers={[restrictToHorizontalAxis]}
      onDragEnd={handleDragEnd}
      sensors={sensors}
    >
      <Table.ScrollArea borderWidth={1}>
        <Table.Root variant="outline">
          <Table.Header>
            {table.getHeaderGroups().map((headerGroup) => (
              <Table.Row key={headerGroup.id}>
                <Table.ColumnHeader w={6}>
                  <Checkbox.Root
                    verticalAlign="middle"
                    size="sm"
                    aria-label="Select all rows"
                    checked={
                      indeterminate ? 'indeterminate' : selection.length > 0
                    }
                    onCheckedChange={(changes) =>
                      setSelection(
                        changes.checked
                          ? table.getRowModel().rows.map((row) => row.id)
                          : [],
                      )
                    }
                  >
                    <Checkbox.HiddenInput />
                    <Checkbox.Control />
                  </Checkbox.Root>
                </Table.ColumnHeader>
                <SortableContext
                  items={columnOrder}
                  strategy={horizontalListSortingStrategy}
                >
                  {headerGroup.headers.map((header) => (
                    <DraggableTableHeader header={header} key={header.id} />
                  ))}
                </SortableContext>
              </Table.Row>
            ))}
          </Table.Header>
          <Table.Body>
            {table.getRowModel().rows.map((row) => (
              <Table.Row key={row.id}>
                <TableCell>
                  <Checkbox.Root
                    size="sm"
                    aria-label="Select row"
                    checked={selection.includes(row.id)}
                    onCheckedChange={(changes) =>
                      setSelection((prev) =>
                        changes.checked
                          ? [...prev, row.id]
                          : selection.filter((id) => id !== row.id),
                      )
                    }
                  >
                    <Checkbox.HiddenInput />
                    <Checkbox.Control />
                  </Checkbox.Root>
                </TableCell>
                {row.getVisibleCells().map((cell) => (
                  <SortableContext
                    key={cell.id}
                    items={columnOrder}
                    strategy={horizontalListSortingStrategy}
                  >
                    <DragAlongTableCell cell={cell} />
                  </SortableContext>
                ))}
              </Table.Row>
            ))}
          </Table.Body>
        </Table.Root>
        <ActionBar.Root open={selection.length > 0}>
          <Portal>
            <ActionBar.Positioner>
              <ActionBar.Content>
                <ActionBar.SelectionTrigger>
                  {selection.length} selected
                </ActionBar.SelectionTrigger>
                <ActionBar.Separator />
                <Button variant="outline" size="sm">
                  Start
                </Button>
                <Button variant="outline" size="sm">
                  Stop
                </Button>
                <Button variant="outline" size="sm">
                  Delete
                </Button>
              </ActionBar.Content>
            </ActionBar.Positioner>
          </Portal>
        </ActionBar.Root>
      </Table.ScrollArea>
    </DndContext>
  );
};

const DraggableTableHeader: FunctionComponent<{
  header: Header<InstanceData, unknown>;
}> = ({ header }) => {
  const { attributes, isDragging, listeners, setNodeRef, transform } =
    useSortable({ id: header.column.id });

  return (
    <Table.ColumnHeader
      cursor="pointer"
      colSpan={header.colSpan}
      onClick={header.column.getToggleSortingHandler()}
      ref={setNodeRef}
      css={{
        '& .sort-icon': {
          opacity: 0,
        },
        '&:hover .sort-icon': {
          opacity: 1,
        },
        position: 'relative',
        transform: CSS.Translate.toString(transform),
        transition: 'width transform 0.2s ease-in-out',
        whiteSpace: 'nowrap',
        width: header.column.getSize(),
        zIndex: isDragging ? 1 : 0,
      }}
    >
      <button
        style={{ verticalAlign: 'middle' }}
        {...attributes}
        {...listeners}
      >
        <PiDotsSixVertical size={18} style={{ cursor: 'pointer' }} />
      </button>
      {header.isPlaceholder ? null : (
        <Span mx={1}>
          {flexRender(header.column.columnDef.header, header.getContext())}
        </Span>
      )}
      <Span ml={1} css={{ '& svg': { display: 'inline' } }}>
        {header.column.getIsSorted() ? (
          header.column.getIsSorted() === 'desc' ? (
            <FaSortDown />
          ) : (
            <FaSortUp />
          )
        ) : (
          <FaSort className="sort-icon" />
        )}
      </Span>
    </Table.ColumnHeader>
  );
};

const DragAlongTableCell: FunctionComponent<{
  cell: Cell<InstanceData, unknown>;
}> = ({ cell }) => {
  const { isDragging, setNodeRef, transform } = useSortable({
    id: cell.column.id,
  });

  const style: CSSProperties = {
    position: 'relative',
    transform: CSS.Translate.toString(transform),
    transition: 'width transform 0.2s ease-in-out',
    width: cell.column.getSize(),
    zIndex: isDragging ? 1 : 0,
  };

  return (
    <Table.Cell ref={setNodeRef} style={style}>
      {flexRender(cell.column.columnDef.cell, cell.getContext())}
    </Table.Cell>
  );
};
