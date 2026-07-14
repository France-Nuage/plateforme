import {
  ConnectionInfoField,
  CreateManagedInstanceInput,
  ManagedService,
  ManagedServiceInstance,
  ManagedServicePlan,
  ManagedServiceVersion,
  UpgradeManagedInstanceInput,
} from '../../models';

export interface ManagedServiceService {
  listServices: () => Promise<ManagedService[]>;
  getService: (slug: string) => Promise<ManagedService>;
  listVersions: (serviceSlug: string) => Promise<ManagedServiceVersion[]>;
  listPlans: (serviceSlug: string) => Promise<ManagedServicePlan[]>;
  createInstance: (
    data: CreateManagedInstanceInput,
  ) => Promise<ManagedServiceInstance>;
  listInstances: (projectSlug: string) => Promise<ManagedServiceInstance[]>;
  getInstance: (instanceId: string) => Promise<ManagedServiceInstance>;
  upgradeInstance: (
    data: UpgradeManagedInstanceInput,
  ) => Promise<ManagedServiceInstance>;
  deleteInstance: (instanceId: string) => Promise<void>;
  getInstanceConnectionInfo: (
    instanceId: string,
  ) => Promise<ConnectionInfoField[]>;
}
