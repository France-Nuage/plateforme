import { hypervisors } from "@/fixtures";
import { HypervisorService } from "./hypervisor.interface";

export class HypervisorMockService implements HypervisorService {
  /** @inheritdoc */
  list() {
    return Promise.resolve(hypervisors(2));
  }

  /** @inheritdoc */
  create() {
    return Promise.resolve();
  }
}

export const hypervisorMockService = new HypervisorMockService();
