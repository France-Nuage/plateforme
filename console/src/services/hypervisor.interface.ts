import { Hypervisor, HypervisorFormValue } from "@/types";

export interface HypervisorService {
  list: () => Promise<Hypervisor[]>;
  register: (value: HypervisorFormValue) => Promise<Hypervisor>;
}
