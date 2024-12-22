import type { HttpContext } from '@adonisjs/core/http'
import region_service from '#services/v1/infrastructure/region_service'
import RegionPolicy from '#policies/infrastructure/region_policy'

export default class RegionsController {
  /**
   * Display a list of resource
   */
  async index({ response, request, bouncer }: HttpContext) {
    await bouncer.with(RegionPolicy).authorize('index')
    return response.ok(await region_service.list(request.qs().includes))
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
    await bouncer.with(RegionPolicy).authorize('show')
    return response.ok(await region_service.get(params.id, request.qs().includes))
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
