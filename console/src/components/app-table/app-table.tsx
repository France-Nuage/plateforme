import { ComponentPropsWithoutRef } from "react";

export type AppTableProps<T> = {
  data: T[];
} & ComponentPropsWithoutRef<"table">;

export const AppTable = <T,>({ data, ...props }: AppTableProps<T>) => (
  <table className="min-w-full divide-y divide-gray-300" {...props}>
    <thead></thead>
    <tbody className="divide-y divide-gray-200">
      {data.map((row, index) => (
        <tr key={index}>
          {Object.values(row as object).map((value, index) => (
            <td key={index} className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
              {String(value)}
            </td>
          ))}
        </tr>
      ))}
    </tbody>
  </table>
);
