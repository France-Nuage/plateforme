import { InstancesClient } from "@/protocol/instances.client";
import {
  InstanceInfo as RpcInstance,
  InstanceStatus as RpcInstanceStatus,
} from "@/protocol/instances";
import { Instance, InstanceStatus } from "@/types";
import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";

export class InstanceService {
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

  public list(): Promise<Instance[]> {
    return this.client
      .listInstances({})
      .response.then(({ instances }) => instances.map(fromRpcInstance));
  }
}

// Converts a protocol Instance into a concrete Instance.
function fromRpcInstance(instance: RpcInstance): Instance {
  return {
    id: instance.id,
    cpuUsagePercent: instance.cpuUsagePercent,
    maxCpuCores: instance.maxCpuCores,
    maxMemoryBytes: instance.maxMemoryBytes.toString(),
    memoryUsageBytes: instance.memoryUsageBytes.toString(),
    name: instance.name,
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
