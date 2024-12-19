import type { HttpContext } from '@adonisjs/core/http'
import db from '@adonisjs/lucid/services/db'
import IAMMemberPolicy from '#policies/iam/iam_member_policy'

const filterSQLKey = {
  organization: 'organization__id',
  folder: 'folder__id',
  project: 'project__id',
}

export default class MembersController {
  /**
   * Display a list of resource
   */
  async index({ response, params, bouncer, request }: HttpContext) {
    await bouncer.with(IAMMemberPolicy).authorize('index')

    const page = request.input('page', 1)
    const limit = request.input('limit', 10)

    const result = await db
      .from('member.users as u')
      .join('iam.user_resource_policy_binding as b', 'u.id', 'b.member__id')
      .join('iam.resource_policy as p', 'b.policy__id', 'p.policy__id')
      .where(`p.${filterSQLKey[params.resource as keyof typeof filterSQLKey]}`, params.resourceId)
      .groupBy('u.email')
      .select('u.email as member')
      .select(db.raw('array_agg(DISTINCT b.role__id) as roles'))
      .paginate(page, limit)

    return response.ok({
      data: {
        bindings: result.all(),
        // etag: "BwWWja0YfJA=",
        // version: 3,
      },
      meta: result.getMeta(),
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
  async show({ response, params, bouncer }: HttpContext) {
    await bouncer.with(IAMMemberPolicy).authorize('show')

    const result = await db
      .from('member.users as u')
      .join('iam.user_resource_policy_binding as b', 'u.id', 'b.member__id')
      .join('iam.resource_policy as p', 'b.policy__id', 'p.policy__id')
      .where(`p.${filterSQLKey[params.resource as keyof typeof filterSQLKey]}`, params.resourceId)
      .andWhere('u.id', params.id)
      .groupBy('u.email')
      .select('u.email as member')
      .select(db.raw('array_agg(DISTINCT b.role__id) as roles'))
      .firstOrFail()

    return response.ok(result)
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
