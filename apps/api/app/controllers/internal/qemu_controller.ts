import { HttpContext } from '@adonisjs/core/http'
import Node from '#models/infrastructure/node'

export default class QemuController {
  async currentStatus({ params, response }: HttpContext) {
    const node = await Node.findOrFail(params.node_id)
    await node.load('cluster')

    const status = await node.api().instance(params.qemu_id).getStatus()

    return response.ok({ status })
  }
}
