import { hypervisor, hypervisors } from "@/fixtures";
import { HypervisorService } from "./hypervisor.interface";

export class HypervisorMockService implements HypervisorService {
  /** @inheritdoc */
  list() {
    return Promise.resolve(hypervisors(2));
  }

  /** @inheritdoc */
  register() {
    return Promise.resolve(hypervisor());
  }
}

export const hypervisorMockService = new HypervisorMockService();
