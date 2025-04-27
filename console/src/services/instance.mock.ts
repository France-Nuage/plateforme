import { instances } from "@/fixtures";
import { InstanceService } from "./instance.interface";

export class InstanceMockService implements InstanceService {
  /** @inheritdoc */
  list() {
    return Promise.resolve(instances(5));
  }
}

export const instanceMockService = new InstanceMockService();
