import { Hypervisor, HypervisorFormValue } from '../models';

export interface HypervisorService {
  list: () => Promise<Hypervisor[]>;
  register: (value: HypervisorFormValue) => Promise<Hypervisor>;
}
