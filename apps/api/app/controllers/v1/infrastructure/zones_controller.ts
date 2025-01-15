import type { HttpContext } from '@adonisjs/core/http'
import zone_service from '#services/v1/infrastructure/zone_service'
import ZonePolicy from '#policies/infrastructure/zone_policy'

export default class ZonesController {
  /**
   * Display a list of resource
   */
  async index({ response, request, bouncer }: HttpContext) {
    await bouncer.with(ZonePolicy).authorize('index')
    return response.ok(await zone_service.list(request.qs().includes))
  }

  /**
   * Handle form submission for the create action
   */
  async store({ response, params, request }: HttpContext) {
    return response.notImplemented({
      params: params,
      request: request,
    })
  }

  /**
   * Show individual record
   */
  async show({ response, request, params, bouncer }: HttpContext) {
    await bouncer.with(ZonePolicy).authorize('show')
    return response.ok(await zone_service.get(params.id, request.qs().includes))
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
}
