import { HttpContext } from '@adonisjs/core/http'
import Node from '#models/infrastructure/node'
import axios from 'axios'

export default class VirtualMachineController {
  async show({ response, params }: HttpContext) {
    const node = await Node.find(params.node_id)

    if (!node) {
      return response.notFound('Resource not found')
    }

    try {
      const hypervisor = await axios.get(
        `${node.url}/api2/json/nodes/${node.name}/qemu/${params.vm_id}`,
        {
          headers: {
            Authorization: node.token,
          },
        }
      )

      return response.ok(hypervisor)
    } catch (e) {
      console.log(e)
      return response.internalServerError(e)
    }
  }
}
