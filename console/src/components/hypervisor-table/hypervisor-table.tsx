import { FunctionComponent } from "react";
import { AppTable, AppTableProps } from "../app-table";
import { Hypervisor } from "@/types";

export type HypervisorTableProps = {
  hypervisors: Hypervisor[];
};

const columns: AppTableProps<Hypervisor>["columns"] = [
  "id",
  "url",
  { key: "storageName", label: "Storage Name" },
];

export const HypervisorTable: FunctionComponent<HypervisorTableProps> = ({
  hypervisors,
}) => <AppTable columns={columns} rows={hypervisors}></AppTable>;
