/**
 * Represents an instance.
 */
export type Instance = {
  // Current CPU utilization as a percentage (0.0-100.0)
  cpuUsagePercent: number;

  // Current disk space utilization (in bytes, cannot exceed max_disk_bytes)
  diskUsageBytes: number;

  // The instance hypervisor id.
  hypervisorId: string;

  // The instance id.
  id: string;

  // The instance ip address.
  ipV4: string;

  // Maximum CPU cores available to the instance (max 99).
  maxCpuCores: number;

  // Maximum disk space available to the instance (in bytes, max 100TB)
  maxDiskBytes: number;

  // Maximum memory available to the instance (in bytes, max 64GB)
  maxMemoryBytes: number;

  // Current memory utilization (in bytes, cannot exceed max_memory_bytes)
  memoryUsageBytes: number;

  // The instance name.
  name: string;

  // The instance project id.
  projectId: string;

  // Current operational status of the instance
  status: InstanceStatus;

  // The instance zero trust network id.
  zeroTrustNetworkId: string | undefined;
};

/**
 * The instance form creation/update value.
 */
export type InstanceFormValue = Pick<
  Instance,
  'maxCpuCores' | 'maxMemoryBytes' | 'name' | 'projectId'
>;

/**
 * Convenience enum for mapping usual amounts of GB to theyr  byte value.
 */
export enum MemoryBytes {
  '1G' = 1 * 2 ** 30,
  '4G' = 4 * 2 ** 30,
  '8G' = 8 * 2 ** 30,
  '12G' = 12 * 2 ** 30,
  '16G' = 16 * 2 ** 30,
  '20G' = 20 * 2 ** 30,
  '24G' = 24 * 2 ** 30,
  '28G' = 28 * 2 ** 30,
  '32G' = 32 * 2 ** 30,
}

/**
 * The instance status variants.
 */
export enum InstanceStatus {
  UndefinedInstanceStatus = 'undefined instance status',
  Running = 'running',
  Stopped = 'stopped',
  Stopping = 'stopping',
  Provisioning = 'provisioning',
  Staging = 'staging',
  Suspended = 'suspended',
  Suspending = 'suspending',
  Terminated = 'terminated',
  Deprovisionning = 'deprovisionning',
  Repairing = 'repairing',
}
