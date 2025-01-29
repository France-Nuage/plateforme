import { HttpContext } from '@adonisjs/core/http'
import { proxmoxApi } from '../../utils/ProxmoxHelper.js'
import Node from '#models/infrastructure/node'

export default class QemuController {
  async currentStatus({ params, response }: HttpContext) {
    const node = await Node.findOrFail(params.node_id)
    const result = await proxmoxApi.node.qemu.status.current({
      url: node.url,
      nodeName: node.name,
      token: node.token,
      vmid: params.qemu_id,
    })

    return response.ok(result)
  }
}
