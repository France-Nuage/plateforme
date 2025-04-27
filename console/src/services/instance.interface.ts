import { Instance } from "@/types";

export interface InstanceService {
  list: () => Promise<Instance[]>;
}
