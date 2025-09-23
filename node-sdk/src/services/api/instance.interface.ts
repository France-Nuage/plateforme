import { Instance, InstanceFormValue } from '../../models';

export interface InstanceService {
  /** Lists the available instances. */
  list: () => Promise<Instance[]>;

  /** Clone a given instance. */
  clone: (id: string) => Promise<Instance>;

  /** Create a new instance. */
  create: (data: InstanceFormValue) => Promise<Instance>;

  /** Remove the given instance. */
  remove: (id: string) => Promise<void>;

  /** Start the given instance. */
  start: (id: string) => Promise<void>;

  /** Stop the given instance. */
  stop: (id: string) => Promise<void>;
}
