import { ComponentPropsWithoutRef, Key } from "react";

export type AppTableProps<T> = {
  columns: (keyof T | { key: keyof T; label: string })[];
  rows: T[];
} & ComponentPropsWithoutRef<"table">;

export const AppTable = <T,>({ columns, rows, ...props }: AppTableProps<T>) => (
  <table className="min-w-full divide-y divide-gray-200" {...props}>
    <thead>
      <tr>
        {columns.map((column) => (
          <th
            className="px-3 py-3.5 text-left text-xs font-medium uppercase tracking-wide text-gray-500"
            key={(typeof column === "object" ? column.key : column) as Key}
          >
            {typeof column === "object" ? column.label : String(column)}
          </th>
        ))}
      </tr>
    </thead>
    <tbody className="divide-y divide-gray-200">
      {rows.map((row, index) => (
        <tr key={index}>
          {columns.map((column) => (
            <td
              key={(typeof column === "object" ? column.key : column) as Key}
              className="whitespace-nowrap px-3 py-4 text-sm text-gray-500"
            >
              {String(row[typeof column === "object" ? column.key : column])}
            </td>
          ))}
        </tr>
      ))}
    </tbody>
  </table>
);
