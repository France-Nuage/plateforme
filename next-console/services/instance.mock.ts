import { instance, instances } from "@/fixtures";
import { InstanceService } from "./instance.interface";
import { InstanceFormValue } from "@/types";

export class InstanceMockService implements InstanceService {
  /** @inheritdoc */
  list() {
    return Promise.resolve(instances(5));
  }

  /** @inheritdoc */
  create(data: InstanceFormValue) {
    return Promise.resolve({ ...instance(), ...data });
  }
}

export const instanceMockService = new InstanceMockService();
