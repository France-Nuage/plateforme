import Node from '#models/infrastructure/node'
import Instance from '#models/infrastructure/instance'
import BootDisk from '#models/infrastructure/boot_disk'

export interface HypervisorApi {
  /**
   * Get the hypervisor actions for the given node
   */
  node(node: Node): {
    /**
     * Get the hypervisor actions for the given instance
     */
    instance(id: Instance['pveVmId']): {
      /**
       * Create a new instance on the given node
       */
      create(config: { name: string }): Promise<void>

      /**
       * Delete the given instance
       */
      delete(): Promise<void>

      /**
       * Get the status of the given instance
       */
      getStatus(): Promise<Instance['status']>

      /**
       * Get the configuration of the given instance
       */
      getConfig(): Promise<HypervisorInstanceConfig>

      /**
       * Start the given instance
       */
      start(): Promise<void>

      /**
       * Start the given instance
       */
      stop(): Promise<void>
    }

    /**
     * List the instances of a node
     */
    listInstances(): Promise<HypervisorInstance[]>
  }

  /**
   * Get the id of the next instance that will be created on the hypervisor
   */
  getNextInstanceId(): Promise<Instance['pveVmId']>

  /**
   * List the nodes of a cluster
   */
  listNodes(): Promise<Pick<Node, 'name' | 'clusterId'>[]>
}

export type HypervisorInstance = Pick<Instance, 'name' | 'nodeId' | 'pveVmId' | 'status'>

export type HypervisorInstanceConfig = {
  disk: Pick<BootDisk, 'os' | 'size' | 'type'>
}
