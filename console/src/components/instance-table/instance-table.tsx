import { FunctionComponent } from "react";
import { AppTable, AppTableProps } from "@/components";
import { Instance } from "@/types";

export type InstanceTableProps = {
  instances: Instance[];
};

const columns: AppTableProps<Instance>["columns"] = [
  "name",
  "status",
  { key: "cpuUsagePercent", label: "CPU Usage" },
  { key: "maxCpuCores", label: "CPU Max" },
  { key: "memoryUsageBytes", label: "Memory Usage" },
  { key: "maxMemoryBytes", label: "Memory Max" },
];

export const InstanceTable: FunctionComponent<InstanceTableProps> = ({
  instances,
}) => <AppTable columns={columns} rows={instances}></AppTable>;
