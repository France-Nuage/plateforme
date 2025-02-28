import Cluster from '#models/infrastructure/cluster'
import Node from '#models/infrastructure/node'
import Instance, { Status } from '#models/infrastructure/instance'
import db from '@adonisjs/lucid/services/db'
import { HypervisorInstance } from '#services/hypervisor/hypervisor.api'
import BootDisk from '#models/infrastructure/boot_disk'
import { randomUUID } from 'node:crypto'

/**
 * Synchronize the given cluster
 *
 * @param cluster The cluster to synchronize
 */
export async function synchronizeCluster(cluster: Cluster) {
  const data = await cluster.api().listNodes()
  const nodes = await Node.updateOrCreateMany(['clusterId', 'name'], data)

  const nodeSynchronizations = nodes.map((node) => synchronizeClusterNode(cluster, node))
  await Promise.all(nodeSynchronizations)
}

/**
 * Synchronize the given node
 *
 * @param cluster The cluster the node belongs to
 * @param node The node to synchronize
 */
export async function synchronizeClusterNode(cluster: Cluster, node: Node) {
  const defaultProjectId = await getClusterDefaultProject(cluster)

  const listToRecord = <T extends Pick<Instance, 'pveVmId'>>(
    record: Record<T['pveVmId'], T>,
    instance: T
  ) => ({ ...record, [instance.pveVmId]: instance })

  // Get the instances from the hypervisor
  const hypervisorInstances = await cluster.api().node(node).listInstances()
  const hypervisorRecord = hypervisorInstances.reduce(
    listToRecord,
    {} as Record<Instance['pveVmId'], HypervisorInstance>
  )

  // Get the instances from the database
  await node.load('instances')
  const databaseRecord = node.instances.reduce(
    listToRecord,
    {} as Record<Instance['pveVmId'], Instance>
  )

  // list all instance ids existing across the database and the hypervisor
  const ids = Object.keys({ ...databaseRecord, ...hypervisorRecord })

  const work = ids.map((id) =>
    synchronizeInstance(cluster, node, databaseRecord[id], hypervisorRecord[id], defaultProjectId)
  )
  await Promise.all(work)
}

/**
 * Synchronize the given instance.
 *
 * @param cluster The cluster the instance belongs to
 * @param node The node the instance belongs to
 * @param existing The existing instance in database, if any
 * @param distant The distant instance in the hypervisor
 * @param defaultProjectId The default project to attach the instance to
 */
async function synchronizeInstance(
  cluster: Cluster,
  node: Node,
  existing: Instance | undefined,
  distant: HypervisorInstance | undefined,
  defaultProjectId: string
): Promise<void> {
  // Prevent attempting instance synchronization with insufficient data
  if (!existing && !distant) {
    throw new Error('Invalid parameters: neither local nor distant instance supplied')
  }

  // Get the hypervisor instance config
  const config = await cluster
    .api()
    .node(node)
    .instance((distant ?? existing)!.pveVmId)
    .getConfig()

  // Update the database copy of the boot disk
  const bootDisk = await BootDisk.updateOrCreate(
    {
      id: existing?.bootDiskId ?? randomUUID(),
    },
    {
      os: config.disk.os,
      size: config.disk.size,
      type: config.disk.type,
    }
  )

  await Instance.updateOrCreate(
    {
      id: existing?.id || randomUUID(),
    },
    {
      ...distant,
      bootDiskId: bootDisk.id,
      projectId: existing?.projectId ?? defaultProjectId,
      // Mark the instance as deleted if it does not exist on the hypervisor anymore
      status: distant?.status ?? Status.Deleted,
    }
  )
}

// TODO: where should this function live?
async function getClusterDefaultProject(cluster: Cluster): Promise<string> {
  const { id } = await db
    .from('resource.projects')
    .join('resource.folders', 'resource.projects.folder__id', 'resource.folders.folder__id')
    .join(
      'resource.organizations',
      'resource.folders.organization__id',
      'resource.organizations.organization__id'
    )
    .join(
      'infrastructure.clusters',
      'resource.organizations.organization__id',
      'infrastructure.clusters.organization__id'
    )
    .where('infrastructure.clusters.cluster__id', cluster.id)
    .where('resource.projects.name', 'Interne')
    .select('resource.projects.project__id AS id')
    .firstOrFail()

  return id
}
