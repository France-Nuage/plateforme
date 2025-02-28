import type { HttpContext } from '@adonisjs/core/http'

export default class ClustersController {
  /**
   * Display a list of resource
   */
  async index({ response, params, request }: HttpContext) {
    return response.notImplemented({
      params: params,
      request: request,
    })
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
  async show({ response, params, request }: HttpContext) {
    return response.notImplemented({
      params: params,
      request: request,
    })
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
