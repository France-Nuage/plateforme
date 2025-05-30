import { Instance, InstanceFormValue } from '@/types';

export interface InstanceService {
  /** Lists the available instances. */
  list: () => Promise<Instance[]>;

  /** Create a new instance. */
  create: (data: InstanceFormValue) => Promise<Instance>;
}
