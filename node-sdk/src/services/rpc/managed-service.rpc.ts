import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import {
  ManagedServiceInstanceProto,
  ManagedServicePlanProto,
  ManagedServiceProto,
  ManagedServiceVersionProto,
} from '../../generated/rpc/managed';
import { ManagedServicesClient } from '../../generated/rpc/managed.client';
import {
  CreateManagedInstanceInput,
  ManagedInstanceStatus,
  ManagedService,
  ManagedServiceInstance,
  ManagedServicePlan,
  ManagedServicePlanEntitlement,
  ManagedServiceVersion,
  UpgradeManagedInstanceInput,
} from '../../models';
import { ManagedServiceService } from '../api';

export class ManagedServiceRpcService implements ManagedServiceService {
  private client: ManagedServicesClient;

  constructor(transport: GrpcWebFetchTransport) {
    this.client = new ManagedServicesClient(transport);
  }

  public createInstance(
    data: CreateManagedInstanceInput,
  ): Promise<ManagedServiceInstance> {
    return this.client
      .createInstance({
        projectId: data.projectId,
        organizationId: data.organizationId,
        serviceSlug: data.serviceSlug,
        versionId: data.versionId,
        planId: data.planId,
        userValues: data.userValues,
        secretValues: data.secretValues,
      })
      .response.then(({ instance }) => fromRpcInstance(instance!));
  }

  public deleteInstance(instanceId: string): Promise<void> {
    return this.client.deleteInstance({ instanceId }).response.then(() => {});
  }

  public getInstance(instanceId: string): Promise<ManagedServiceInstance> {
    return this.client
      .getInstance({ instanceId })
      .response.then(({ instance }) => fromRpcInstance(instance!));
  }

  public getService(slug: string): Promise<ManagedService> {
    return this.client
      .getService({ slug })
      .response.then(({ service }) => fromRpcService(service!));
  }

  public listInstances(projectId: string): Promise<ManagedServiceInstance[]> {
    return this.client
      .listInstances({ projectId })
      .response.then(({ instances }) => instances.map(fromRpcInstance));
  }

  public listServices(): Promise<ManagedService[]> {
    return this.client
      .listServices({})
      .response.then(({ services }) => services.map(fromRpcService));
  }

  public listVersions(serviceSlug: string): Promise<ManagedServiceVersion[]> {
    return this.client
      .listVersions({ serviceSlug })
      .response.then(({ versions }) => versions.map(fromRpcVersion));
  }

  public listPlans(serviceSlug: string): Promise<ManagedServicePlan[]> {
    return this.client
      .listPlans({ serviceSlug })
      .response.then(({ plans }) => plans.map(fromRpcPlan));
  }

  public upgradeInstance(
    data: UpgradeManagedInstanceInput,
  ): Promise<ManagedServiceInstance> {
    return this.client
      .upgradeInstance({
        instanceId: data.instanceId,
        versionId: data.versionId,
        userValues: data.userValues,
        secretValues: data.secretValues,
      })
      .response.then(({ instance }) => fromRpcInstance(instance!));
  }
}

function fromRpcService(service: ManagedServiceProto): ManagedService {
  return {
    id: service.id,
    slug: service.slug,
    name: service.name,
    description: service.description,
    category: service.category,
    databaseEngine: service.databaseEngine,
    iconUrl: service.iconUrl,
    createdAt: service.createdAt
      ? new Date(Number(service.createdAt.seconds) * 1000).toISOString()
      : new Date().toISOString(),
  };
}

function fromRpcVersion(
  version: ManagedServiceVersionProto,
): ManagedServiceVersion {
  return {
    id: version.id,
    serviceId: version.serviceId,
    chartVersion: version.chartVersion,
    appVersion: version.appVersion,
    ociReference: version.ociReference,
    configurableValuesSchema: version.configurableValuesSchema,
    uiSchema: version.uiSchema,
    createdAt: version.createdAt
      ? new Date(Number(version.createdAt.seconds) * 1000).toISOString()
      : new Date().toISOString(),
  };
}

function fromRpcPlan(plan: ManagedServicePlanProto): ManagedServicePlan {
  return {
    id: plan.id,
    serviceId: plan.serviceId,
    slug: plan.slug,
    name: plan.name,
    description: plan.description,
    status: plan.status as 'active' | 'archived',
    highlighted: plan.highlighted,
    valuesOverride: plan.valuesOverride,
    entitlements: plan.entitlements.map(
      (e): ManagedServicePlanEntitlement => ({
        key: e.key,
        label: e.label,
        value: e.value,
      }),
    ),
    priceMonthlyCents: plan.priceMonthlyCents
      ? Number(plan.priceMonthlyCents)
      : undefined,
    priceYearlyCents: plan.priceYearlyCents
      ? Number(plan.priceYearlyCents)
      : undefined,
    createdAt: plan.createdAt
      ? new Date(Number(plan.createdAt.seconds) * 1000).toISOString()
      : new Date().toISOString(),
  };
}

function fromRpcInstance(
  instance: ManagedServiceInstanceProto,
): ManagedServiceInstance {
  const statusMap: Record<string, ManagedInstanceStatus> = {
    provisioning: ManagedInstanceStatus.Provisioning,
    running: ManagedInstanceStatus.Running,
    upgrading: ManagedInstanceStatus.Upgrading,
    failed: ManagedInstanceStatus.Failed,
    deleting: ManagedInstanceStatus.Deleting,
    deleted: ManagedInstanceStatus.Deleted,
  };

  return {
    id: instance.id,
    serviceId: instance.serviceId,
    versionId: instance.versionId,
    planId: instance.planId,
    projectId: instance.projectId,
    organizationId: instance.organizationId,
    namespace: instance.namespace,
    releaseName: instance.releaseName,
    userValues: instance.userValues,
    status: statusMap[instance.status] ?? ManagedInstanceStatus.Provisioning,
    createdAt: instance.createdAt
      ? new Date(Number(instance.createdAt.seconds) * 1000).toISOString()
      : new Date().toISOString(),
  };
}
