import { FunctionComponent } from "react";
import { AppButton, AppCard, AppTable, AppTableProps } from "@/components";
import { Hypervisor } from "@/types";

export type HypervisorTableProps = {
  hypervisors: Hypervisor[];
  onClick: () => void;
};

const columns: AppTableProps<Hypervisor>["columns"] = [
  "id",
  "url",
  { key: "storageName", label: "Storage Name" },
];

export const HypervisorTable: FunctionComponent<HypervisorTableProps> = ({
  hypervisors,
  onClick,
}) =>
  <AppCard header={
    <div className="flex justify-between items-center">
      <h2>Hyperviseurs</h2>
      <AppButton onClick={onClick}>Ajouter un hyperviseur</AppButton>
    </div>
  } padded={false}>
    <AppTable columns={columns} rows={hypervisors}></AppTable>
  </AppCard>;
