import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';
import { RpcError } from '@protobuf-ts/runtime-rpc';
import { toast } from 'react-toastify';

import {
  Instance as RpcInstance,
  InstanceStatus as RpcInstanceStatus,
} from '@/generated/rpc/instances';
import { InstancesClient } from '@/generated/rpc/instances.client';
import { Instance, InstanceFormValue, InstanceStatus } from '@/types';

import { InstanceService } from './instance.interface';
import { transport } from './transport.rpc';

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
  public create(data: InstanceFormValue): Promise<Instance> {
    return this.client
      .createInstance({
        cpuCores: data.maxCpuCores,
        image: '',
        memoryBytes: BigInt(data.maxMemoryBytes),
        name: data.name,
        snippet: '',
      })
      .response.then(({ instance }) => fromRpcInstance(instance!));
  }

  /** @inheritdoc */
  public list(): Promise<Instance[]> {
    return this.client
      .listInstances({})
      .response.then(({ instances }) => instances.map(fromRpcInstance))
      .catch((error: RpcError) => {
        toast.error(error.toString());
        return [];
      });
  }
}

export const instanceRpcService = new InstanceRpcService(transport);

// Converts a protocol Instance into a concrete Instance.
function fromRpcInstance(instance: RpcInstance): Instance {
  return {
    cpuUsagePercent: instance.cpuUsagePercent,
    id: instance.id,
    maxCpuCores: instance.maxCpuCores,
    maxMemoryBytes: Number(instance.maxMemoryBytes),
    memoryUsageBytes: Number(instance.memoryUsageBytes),
    name: instance.name,
    projectId: instance.projectId,
    status: fromRpcInstanceStatus(instance.status),
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
