import { Hypervisor, HypervisorFormValue } from "@/types";

export interface HypervisorService {
  list: () => Promise<Hypervisor[]>;
  create: (value: HypervisorFormValue) => Promise<void>;
}
