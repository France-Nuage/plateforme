import { instance, instances } from '@/fixtures';
import { InstanceFormValue } from '@/types';

import { InstanceService } from './instance.interface';

export class InstanceMockService implements InstanceService {
  /** @inheritdoc */
  create(data: InstanceFormValue) {
    return Promise.resolve({ ...instance(), ...data });
  }

  /** @inheritdoc */
  list() {
    return Promise.resolve(instances(5));
  }
}

export const instanceMockService = new InstanceMockService();
