import { HypervisorApi } from '#services/hypervisor/hypervisor.api'
import proxmoxApi, { Proxmox } from 'proxmox-api'
import Cluster from '#models/infrastructure/cluster'
import { Status } from '#models/infrastructure/instance'

/**
 * Convert the proxmox API to our normalized Hypervisor API.
 *
 * @see https://github.com/UrielCh/proxmox-api
 */
const proxmoxApiToHypervisorApi = (api: Proxmox.Api, cluster: Cluster): HypervisorApi => ({
  /**
   * @inheritdoc
   */
  node: (node) => ({
    /**
     * @inheritdoc
     */
    instance: (id) => ({
      /**
       * @inheritdoc
       */
      async create({ ...config }) {
        api.nodes.$(node.name).qemu.$post({ ...config, vmid: Number(id) })
      },

      /**
       * @inheritdoc
       */
      async delete() {
        await api.nodes.$(node.name).qemu.$(Number(id)).$delete()
      },

      /**
       * @inheritdoc
       */
      async getStatus() {
        const response = await api.nodes.$(node.name).qemu.$(Number(id)).status.current.$get()

        return (response.status as 'running' | 'stopped').toUpperCase() as Status
      },

      /**
       * @inheritdoc
       */
      async getConfig() {
        const response = await api.nodes.$(node.name).qemu.$(Number(id)).config.$get()
        const bootDiskId = response.boot!.match(/order=([^;]+)/)![1]
        const disk = response[bootDiskId] as typeof response.scsi0
        const diskSize = disk!.match(/size=([^G,]+)/)![1]
        const diskType = disk!.includes('ssd') ? 'SSD' : 'HDD'

        return {
          disk: {
            os: response.ostype!,
            size: diskSize,
            type: diskType,
          },
        }
      },

      /**
       * @inheritdoc
       */
      async start() {
        await api.nodes.$(node.name).qemu.$(Number(id)).status.start.$post()
      },

      /**
       * @inheritdoc
       */
      async stop() {
        await api.nodes.$(node.name).qemu.$(Number(id)).status.start.$post()
      },
    }),

    /**
     * @inheritdoc
     */
    async listInstances() {
      const response = await api.nodes.$(node.name).qemu.$get()

      return response.map((item) => ({
        name: item.name!,
        nodeId: node.id,
        pveVmId: item.vmid.toString(),
        status: item.status.toUpperCase() as Status,
      }))
    },
  }),

  /**
   * @inheritdoc
   */
  async getNextInstanceId() {
    const response = await api.cluster.nextid.$get()

    return response.toString()
  },

  /**
   * @inheritdoc
   */
  async listNodes() {
    const response = await api.nodes.$get()

    return response.map((item) => ({
      clusterId: cluster.id,
      name: item.node,
    }))
  },
})

/**
 * Get a HypervisorApi instance for the given cluster.
 *
 * NOTE: this function assumes the given cluster is a proxmox cluster.
 */
export function getProxmoxClusterHypervisorApi(cluster: Cluster): HypervisorApi {
  const api = proxmoxApi({
    host: cluster.host,
    tokenID: cluster.tokenId,
    tokenSecret: cluster.tokenSecret,
  })

  return proxmoxApiToHypervisorApi(api, cluster)
}
