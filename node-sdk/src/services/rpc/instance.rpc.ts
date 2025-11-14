import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import {
  Instance as RpcInstance,
  InstanceStatus as RpcInstanceStatus,
} from '../../generated/rpc/compute';
import { InstancesClient } from '../../generated/rpc/compute.client';
import { Instance, InstanceFormValue, InstanceStatus } from '../../models';
import { InstanceService } from '../api';

export class InstanceRpcService implements InstanceService {
  /**
   * The gRPC instances client.
   */
  private client: InstancesClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new InstancesClient(transport);
  }

  /** @inheritdoc */
  public clone(id: string) {
    return this.client
      .clone({ id })
      .response.then((data) => fromRpcInstance(data));
  }

  /** @inheritdoc */
  public create(data: InstanceFormValue): Promise<Instance> {
    return this.client
      .create({
        cpuCores: data.maxCpuCores,
        diskBytes: BigInt(data.maxDiskBytes),
        image: data.image,
        memoryBytes: BigInt(data.maxMemoryBytes),
        name: data.name,
        projectId: data.projectId,
        snippet: data.snippet,
      })
      .response.then(({ instance }) => fromRpcInstance(instance!));
  }

  /** @inheritdoc */
  public list(): Promise<Instance[]> {
    return this.client
      .list({})
      .response.then(({ instances }) => instances.map(fromRpcInstance));
  }

  /** @inheritdoc */
  public remove(id: string) {
    return this.client.delete({ id }).response.then(() => {});
  }

  /** @inheritdoc */
  public start(id: string) {
    return this.client.start({ id }).then(() => {});
  }

  /** @inheritdoc */
  public stop(id: string) {
    return this.client.stop({ id }).response.then(() => {});
  }
}

// Converts a protocol Instance into a concrete Instance.
function fromRpcInstance(instance: RpcInstance): Instance {
  return {
    cpuUsagePercent: instance.cpuUsagePercent,
    createdAt: (instance.createdAt
      ? new Date(Number(instance.createdAt!.seconds))
      : new Date()
    ).toISOString(),
    diskUsageBytes: Number(instance.diskUsageBytes),
    hypervisorId: instance.hypervisorId,
    id: instance.id,
    ipV4: instance.ipV4,
    maxCpuCores: instance.maxCpuCores,
    maxDiskBytes: Number(instance.maxDiskBytes),
    maxMemoryBytes: Number(instance.maxMemoryBytes),
    memoryUsageBytes: Number(instance.memoryUsageBytes),
    name: instance.name,
    projectId: instance.projectId,
    status: fromRpcInstanceStatus(instance.status),
    updatedAt: (instance.updatedAt
      ? new Date(Number(instance.updatedAt!.seconds))
      : new Date()
    ).toISOString(),
    zeroTrustNetworkId: instance.zeroTrustNetworkId,
  };
}

// Converts a protocol InstanceStatus into a concrete InstanceStatus.
function fromRpcInstanceStatus(status: RpcInstanceStatus): InstanceStatus {
  return {
    [RpcInstanceStatus.UNDEFINED_INSTANCE_STATUS]:
      InstanceStatus.UndefinedInstanceStatus,
    [RpcInstanceStatus.RUNNING]: InstanceStatus.Running,
    [RpcInstanceStatus.STOPPED]: InstanceStatus.Stopped,
    [RpcInstanceStatus.STOPPING]: InstanceStatus.Stopping,
    [RpcInstanceStatus.PROVISIONING]: InstanceStatus.Provisioning,
    [RpcInstanceStatus.STAGING]: InstanceStatus.Staging,
    [RpcInstanceStatus.SUSPENDED]: InstanceStatus.Suspended,
    [RpcInstanceStatus.SUSPENDING]: InstanceStatus.Suspending,
    [RpcInstanceStatus.TERMINATED]: InstanceStatus.Terminated,
    [RpcInstanceStatus.DEPROVISIONING]: InstanceStatus.Deprovisionning,
    [RpcInstanceStatus.REPAIRING]: InstanceStatus.Repairing,
  }[status];
}
