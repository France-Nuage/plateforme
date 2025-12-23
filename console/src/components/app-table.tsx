import {
  ActionBar,
  Button,
  Checkbox,
  Flex,
  Input,
  Portal,
  Select,
  Span,
  Table,
  TableCell,
  Text,
  createListCollection,
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
  ColumnDef,
  Header,
  SortingState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { CSSProperties, ReactNode, useState } from 'react';
import { FaSort, FaSortDown, FaSortUp } from 'react-icons/fa';
import { PiDotsSixVertical } from 'react-icons/pi';

import { useUrlState } from '@/hooks';

export type AppTableProps<T, U> = {
  columns: ColumnDef<T, U>[];
  data: T[];
  bulkActions?: (selectedRows: T[]) => ReactNode;
};

export const AppTable = <T, U>({
  columns,
  data,
  bulkActions,
}: AppTableProps<T, U>): ReactNode => {
  // Initialize with all columns displayed by default
  const allColumnIds = columns.map((column) => column.id!).join(',');
  const [urlcolumns, setColumns] = useUrlState('columns', allColumnIds);
  const activeColumns = urlcolumns.split(',');

  // Define the bulk edit state
  const [bulkEdit, setBulkEdit] = useState(false);

  // Add search state for filtering
  const [globalFilter, setGlobalFilter] = useUrlState('globalFilter', '');

  // Track the sort column.
  const [sorting, setSorting] = useState<SortingState>([]);

  // Create the react table
  const table = useReactTable({
    columns,
    data,
    enableRowSelection: true,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getSortedRowModel: getSortedRowModel(),
    globalFilterFn: 'includesString',
    onGlobalFilterChange: setGlobalFilter,
    onSortingChange: setSorting,
    state: { globalFilter, sorting },
  });

  // Handle column reordering after drag & drop
  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (active && over && active.id !== over.id) {
      const currentOrder = table.getAllColumns().map((col) => col.id);
      const oldIndex = currentOrder.indexOf(active.id as string);
      const newIndex = currentOrder.indexOf(over.id as string);
      table.setColumnOrder(arrayMove(currentOrder, oldIndex, newIndex));
    }
  };

  // Define the sensors for moving the columns
  const sensors = useSensors(
    useSensor(MouseSensor, {}),
    useSensor(TouchSensor, {}),
    useSensor(KeyboardSensor, {}),
  );

  // Track the displayed columns
  const collection = createListCollection({
    items: table.getAllColumns().map((c) => c.id),
  });

  return (
    <>
      <Flex direction="column" gap={4}>
        <Flex justifyContent="space-between">
          <Input
            placeholder="Rechercher dans toutes les colonnes..."
            value={globalFilter}
            onChange={(e) => setGlobalFilter(e.target.value)}
            maxW="400px"
          />
          <Flex gap={2}>
            <Button
              colorPalette="gray"
              onClick={() => setBulkEdit(!bulkEdit)}
              variant="outline"
            >
              {bulkEdit
                ? "Désactiver l'édition multiple"
                : "Activer l'édition multiple"}
            </Button>
            <Select.Root
              multiple
              collection={collection}
              value={activeColumns}
              onValueChange={(e) => setColumns(e.value.join(','))}
              maxW={60}
            >
              <Select.Control>
                <Select.Trigger>
                  <Select.ValueText placeholder="Colonnes à afficher" />
                </Select.Trigger>
                <Select.IndicatorGroup>
                  <Select.Indicator />
                </Select.IndicatorGroup>
              </Select.Control>
              <Portal>
                <Select.Positioner>
                  <Select.Content>
                    {table
                      .getAllColumns()
                      .filter((column) => column.getCanHide())
                      .map((column) => (
                        <Select.Item item={column.id} key={column.id}>
                          {`${column.columnDef.header}`}
                          <Select.ItemIndicator />
                        </Select.Item>
                      ))}
                  </Select.Content>
                </Select.Positioner>
              </Portal>
            </Select.Root>
          </Flex>
        </Flex>
      </Flex>
      {table.getRowModel().rows.length ? (
        <DndContext
          collisionDetection={closestCenter}
          modifiers={[restrictToHorizontalAxis]}
          onDragEnd={handleDragEnd}
          sensors={sensors}
        >
          <Table.ScrollArea borderWidth={1} maxHeight="calc(100vh - 250px)">
            <Table.Root variant="outline">
              <Table.Header position="sticky" top={0} zIndex={10} bg="bg">
                {table.getHeaderGroups().map((headerGroup) => (
                  <Table.Row key={headerGroup.id}>
                    {bulkEdit && (
                      <Table.ColumnHeader w={6}>
                        <Checkbox.Root
                          verticalAlign="middle"
                          size="sm"
                          aria-label="Select all rows"
                          checked={table.getIsAllRowsSelected()}
                          onCheckedChange={({ checked }) =>
                            table.toggleAllRowsSelected(!!checked)
                          }
                        >
                          <Checkbox.HiddenInput />
                          <Checkbox.Control />
                        </Checkbox.Root>
                      </Table.ColumnHeader>
                    )}
                    <SortableContext
                      items={table.getAllColumns().map((col) => col.id)}
                      strategy={horizontalListSortingStrategy}
                    >
                      {headerGroup.headers
                        .filter((header) =>
                          activeColumns.includes(header.column.id),
                        )
                        .map((header) => (
                          <DraggableTableHeader
                            header={header}
                            key={header.id}
                          />
                        ))}
                    </SortableContext>
                  </Table.Row>
                ))}
              </Table.Header>
              <Table.Body>
                {table.getRowModel().rows.map((row) => (
                  <Table.Row key={row.id}>
                    {bulkEdit && (
                      <TableCell>
                        <Checkbox.Root
                          size="sm"
                          aria-label="Select row"
                          checked={row.getIsSelected()}
                          onCheckedChange={({ checked }) =>
                            row.toggleSelected(!!checked)
                          }
                        >
                          <Checkbox.HiddenInput />
                          <Checkbox.Control />
                        </Checkbox.Root>
                      </TableCell>
                    )}
                    {row
                      .getVisibleCells()
                      .filter((cell) => activeColumns.includes(cell.column.id))
                      .map((cell) => (
                        <SortableContext
                          key={cell.id}
                          items={table.getAllColumns().map((col) => col.id)}
                          strategy={horizontalListSortingStrategy}
                        >
                          <DragAlongTableCell cell={cell} />
                        </SortableContext>
                      ))}
                  </Table.Row>
                ))}
              </Table.Body>
            </Table.Root>
            {bulkActions && (
              <ActionBar.Root
                open={table.getSelectedRowModel().rows.length > 0}
              >
                <Portal>
                  <ActionBar.Positioner>
                    <ActionBar.Content>
                      <ActionBar.SelectionTrigger>
                        {table.getSelectedRowModel().rows.length} selected
                      </ActionBar.SelectionTrigger>
                      <ActionBar.Separator />
                      {bulkActions(
                        table
                          .getSelectedRowModel()
                          .rows.map((row) => row.original),
                      )}
                    </ActionBar.Content>
                  </ActionBar.Positioner>
                </Portal>
              </ActionBar.Root>
            )}
          </Table.ScrollArea>
        </DndContext>
      ) : (
        <Flex
          alignItems="center"
          borderWidth={1}
          flexGrow={1}
          justifyContent="center"
        >
          <Text color="fg.muted">
            {globalFilter
              ? `Aucune donnée ne correspond à la recherche "${globalFilter}"`
              : 'Aucune donnée'}
          </Text>
        </Flex>
      )}
    </>
  );
};

const DraggableTableHeader = <T,>({
  header,
}: {
  header: Header<T, unknown>;
}): ReactNode => {
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

const DragAlongTableCell = <T,>({
  cell,
}: {
  cell: Cell<T, unknown>;
}): ReactNode => {
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
