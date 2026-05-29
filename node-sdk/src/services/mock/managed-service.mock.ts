import {
  managedService,
  managedServiceInstance,
  managedServiceInstances,
  managedServiceVersion,
  managedServices,
} from '../../fixtures';
import {
  CreateManagedInstanceInput,
  UpgradeManagedInstanceInput,
} from '../../models';
import { ManagedServiceService } from '../api';

export class ManagedServiceMockService implements ManagedServiceService {
  createInstance(data: CreateManagedInstanceInput) {
    return Promise.resolve(
      managedServiceInstance({
        projectId: data.projectId,
        organizationId: data.organizationId,
        serviceId: data.serviceSlug,
        versionId: data.versionId,
      }),
    );
  }

  deleteInstance() {
    return Promise.resolve();
  }

  getInstance(instanceId: string) {
    return Promise.resolve(managedServiceInstance({ id: instanceId }));
  }

  getService(slug: string) {
    return Promise.resolve(managedService({ slug, name: slug }));
  }

  listInstances(projectId: string) {
    return Promise.resolve(managedServiceInstances(3, { projectId }));
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
