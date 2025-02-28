import type { HttpContext } from '@adonisjs/core/http'
import role_service from '#services/v1/iam/role_service'
import IAMRolePolicy from '#policies/iam/iam_role_policy'

export default class RolesController {
  /**
   * Display a list of resource
   */
  async index({ response, request, bouncer }: HttpContext) {
    await bouncer.with(IAMRolePolicy).authorize('index')
    const roles = await role_service.list(request.qs().includes)

    return response.ok(roles)
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
  async show({ response, params, request, bouncer }: HttpContext) {
    await bouncer.with(IAMRolePolicy).authorize('index')
    const role = await role_service.get(params.id, request.qs().includes)

    return response.ok(role)
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
