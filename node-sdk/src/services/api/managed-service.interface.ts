import {
  CreateManagedInstanceInput,
  ManagedService,
  ManagedServiceInstance,
  ManagedServiceVersion,
  UpgradeManagedInstanceInput,
} from '../../models';

export interface ManagedServiceService {
  listServices: () => Promise<ManagedService[]>;
  getService: (slug: string) => Promise<ManagedService>;
  listVersions: (serviceSlug: string) => Promise<ManagedServiceVersion[]>;
  createInstance: (
    data: CreateManagedInstanceInput,
  ) => Promise<ManagedServiceInstance>;
  listInstances: (projectId: string) => Promise<ManagedServiceInstance[]>;
  getInstance: (instanceId: string) => Promise<ManagedServiceInstance>;
  upgradeInstance: (
    data: UpgradeManagedInstanceInput,
  ) => Promise<ManagedServiceInstance>;
  deleteInstance: (instanceId: string) => Promise<void>;
}
