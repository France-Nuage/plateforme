import type { HttpContext } from '@adonisjs/core/http'
import InstancePolicy from '#policies/infrastructure/instance_policy'
import instance_service from '#services/v1/infrastructure/instance_service'
import {
  createInstanceValidator,
  getInstanceCurrentPriceValidator,
} from '#validators/v1/infrastructure/instance'

export default class InstancesController {
  /**
   * Display a list of resource
   */
  async index({ response, request, bouncer }: HttpContext) {
    await bouncer.with(InstancePolicy).authorize('index')
    return response.ok(await instance_service.list(request.qs().includes))
  }

  /**
   * Handle form submission for the create action
   */
  async store({ response, request, bouncer }: HttpContext) {
    await bouncer.with(InstancePolicy).authorize('store')
    const payload = await request.validateUsing(createInstanceValidator)
    const instance = await instance_service.create({ ...payload })
    return response.created(instance)
  }

  /**
   * Show individual record
   */
  async show({ response, params, request, bouncer }: HttpContext) {
    await bouncer.with(InstancePolicy).authorize('show')
    return response.ok(await instance_service.get(params.id, request.qs().includes))
  }

  /**
   * Handle form submission for the edit action
   */
  async update({ response, params, request }: HttpContext) {
    return response.notImplemented({
      params: params,
      request: request,
    })
  }

  /**
   * Delete record
   */
  async destroy({ response, params, request }: HttpContext) {
    return response.notImplemented({
      params: params,
      request: request,
    })
  }

  async getPrice({ response, request }: HttpContext) {
    const payload = await request.validateUsing(getInstanceCurrentPriceValidator)

    return response.ok(await instance_service.getCurrentPrice({ ...payload }))
  }
}

// import { createMachine } from "xstate";
//
// export const machine = createMachine({
//   context: {},
//   id: "status",
//   initial: "Init",
//   states: {
//     Init: {
//       on: {
//         created: {
//           target: "Created",
//         },
//       },
//       description:
//         "The initial state where the status is not yet created. The only transition from this state is to the Created state.",
//     },
//     Created: {
//       on: {
//         running: {
//           target: "Running",
//         },
//         stop: {
//           target: "Stop",
//         },
//         restart: {
//           target: "Restart",
//         },
//         delete: {
//           target: "Delete",
//         },
//       },
//       description:
//         "The created state where the status is set up. It can transition to Running, Stop, or Restart.",
//     },
//     Running: {
//       on: {
//         stop: {
//           target: "Stop",
//         },
//         restart: {
//           target: "Restart",
//         },
//         delete: {
//           target: "Delete",
//         },
//       },
//       description:
//         "The running state where the status is actively executing. It can transition to Stop, Restart, or remain in Running.",
//     },
//     Stop: {
//       on: {
//         delete: {
//           target: "Delete",
//         },
//         restart: {
//           target: "Restart",
//         },
//       },
//       description:
//         "The stop state where the status is halted. It cannot transition to any other state.",
//     },
//     Restart: {
//       on: {
//         running: {
//           target: "Running",
//         },
//         stop: {
//           target: "Stop",
//         },
//         delete: {
//           target: "Delete",
//         },
//       },
//       description:
//         "The restart state where the status is reset and can transition to Running or Stop.",
//     },
//     Delete: {
//       type: "final",
//       description:
//         "The restart state where the status is reset and can transition to Running or Stop.",
//     },
//   },
// }).withConfig({});
