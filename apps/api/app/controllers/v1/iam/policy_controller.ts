import { HttpContext } from '@adonisjs/core/http'
import IAMPolicyPolicy from '#policies/iam/iam_policy_policy'
import db from '@adonisjs/lucid/services/db'

const filterSQLKey = {
  organization: 'organization__id',
  folder: 'folder__id',
  project: 'project__id',
}

export default class PoliciesController {
  async index({ response, bouncer, params, request }: HttpContext) {
    await bouncer.with(IAMPolicyPolicy).authorize('index')

    const page = request.input('page', 1)
    const limit = request.input('limit', 10)

    const result = await db
      .from('member.users as u')
      .join('iam.user_resource_policy_binding as b', 'u.id', 'b.member__id')
      .join('iam.resource_policy as p', 'b.policy__id', 'p.policy__id')
      .where(`p.${filterSQLKey[params.resource as keyof typeof filterSQLKey]}`, params.resourceId)
      .groupBy('b.role__id')
      .select('b.role__id')
      .select(db.raw('array_agg(DISTINCT u.email) as members'))
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
   * Show individual record
   */
  async show({ response, params, bouncer }: HttpContext) {
    await bouncer.with(IAMPolicyPolicy).authorize('show')

    const roleId = params.id.replace('%2F', '/')

    const result = await db
      .from('member.users as u')
      .join('iam.user_resource_policy_binding as b', 'u.id', 'b.member__id')
      .join('iam.resource_policy as p', 'b.policy__id', 'p.policy__id')
      .where(`p.${filterSQLKey[params.resource as keyof typeof filterSQLKey]}`, params.resourceId)
      .andWhere('b.role__id', roleId)
      .groupBy('b.role__id')
      .select('b.role__id')
      .select(db.raw('array_agg(DISTINCT u.email) as members'))
      .firstOrFail()

    return response.ok(result)
  }
}
