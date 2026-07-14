import {
  managedService,
  managedServiceInstance,
  managedServiceInstances,
  managedServicePlans,
  managedServiceVersion,
  managedServices,
} from '../../fixtures';
import {
  ConnectionInfoField,
  CreateManagedInstanceInput,
  ManagedServicePlan,
  UpgradeManagedInstanceInput,
} from '../../models';
import { ManagedServiceService } from '../api';

export class ManagedServiceMockService implements ManagedServiceService {
  createInstance(data: CreateManagedInstanceInput) {
    return Promise.resolve(
      managedServiceInstance({
        projectSlug: data.projectSlug,
        organizationSlug: data.organizationSlug,
        serviceId: data.serviceSlug,
        versionId: data.versionId,
        planId: data.planId,
      }),
    );
  }

  deleteInstance() {
    return Promise.resolve();
  }

  getInstance(instanceId: string) {
    return Promise.resolve(managedServiceInstance({ id: instanceId }));
  }

  getInstanceConnectionInfo(): Promise<ConnectionInfoField[]> {
    return Promise.resolve([
      {
        key: 'host',
        label: 'Hote',
        value: 'postgres-rw.ns.svc.cluster.local',
        display: 'text' as const,
        order: 1,
      },
      {
        key: 'port',
        label: 'Port',
        value: '5432',
        display: 'text' as const,
        order: 2,
      },
      {
        key: 'username',
        label: 'Utilisateur',
        value: 'app',
        display: 'text' as const,
        order: 3,
      },
      {
        key: 'password',
        label: 'Mot de passe',
        value: 'mock-password',
        display: 'password' as const,
        order: 4,
      },
    ]);
  }

  getService(slug: string) {
    return Promise.resolve(managedService({ slug, name: slug }));
  }

  listInstances(projectSlug: string) {
    return Promise.resolve(managedServiceInstances(3, { projectSlug }));
  }

  listPlans(serviceSlug: string): Promise<ManagedServicePlan[]> {
    return Promise.resolve(managedServicePlans(2, { serviceId: serviceSlug }));
  }

  listServices() {
    return Promise.resolve(managedServices(6));
  }

  listVersions(serviceSlug: string) {
    return Promise.resolve(
      [...Array(3)].map(() =>
        managedServiceVersion({ serviceId: serviceSlug }),
      ),
    );
  }

  upgradeInstance(data: UpgradeManagedInstanceInput) {
    return Promise.resolve(
      managedServiceInstance({
        id: data.instanceId,
        versionId: data.versionId,
      }),
    );
  }
}

export const managedServiceMockService = new ManagedServiceMockService();
