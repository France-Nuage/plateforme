import { FunctionComponent } from "react";
import { AppButton, AppCard, AppTable, AppTableProps } from "@/components";
import { Instance } from "@/types";

export type InstanceTableProps = {
  instances: Instance[];
  onClick: () => void;
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
  onClick,
}) =>
  <AppCard header={
    <div className="flex justify-between items-center">
      <h2>Instances</h2>
      <AppButton onClick={onClick}>Ajouter une instance</AppButton>
    </div>
  } padded={false}>
    <AppTable columns={columns} rows={instances}></AppTable>
  </AppCard>;
