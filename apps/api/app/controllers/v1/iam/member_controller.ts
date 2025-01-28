import type {HttpContext} from '@adonisjs/core/http'
import db from '@adonisjs/lucid/services/db'
import IAMMemberPolicy from '#policies/iam/iam_member_policy'
import { createMemberValidator } from '#validators/v1/iam/member'
import Policy from '#models/iam/policy'
import User from '#models/user'
import Role from '#models/iam/role'
import Binding from '#models/iam/binding'
import mail from '@adonisjs/mail/services/main'
import env from '#start/env'
import Organization from '#models/resource/organization'
import Folder from '#models/resource/folder'
import Project from '#models/resource/project'

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
  async store({ response, params, request, bouncer, resources }: HttpContext) {
    await bouncer.with(IAMMemberPolicy).authorize('index')

    const payload = await request.validateUsing(createMemberValidator)
    const trx = await db.transaction()
    const user = await User.findByOrFail({ email: payload.email })

    let bodyCreatePolicy = {}
    const resourceIdentifier = {
      organization: {
        modelId: 'organizationId',
        label: "l'organisation",
        model: Organization,
      },
      folder: { modelId: 'folderId', label: 'la filial', model: Folder },
      project: { modelId: 'projectId', label: 'le projet', model: Project },
    }

    resources.forEach((resource) => {
      if (resource.type)
        bodyCreatePolicy = Object.assign(bodyCreatePolicy, {
          [resourceIdentifier[resource.type].modelId]: resource.id,
        })
    })
    try {
      const policy = await Policy.create(bodyCreatePolicy, { client: trx })

      for (const item of payload.roles) {
        const role = await Role.findOrFail(item)

        await Binding.create(
          {
            roleId: role.id,
            memberId: user.id,
            policyId: policy.id,
          },
          {
            client: trx,
          }
        )
      }
      await trx.commit()
    } catch (error) {
      await trx.rollback()

      return response.badRequest({
        message: error.message,
      })
    }

    const resource = await resourceIdentifier[
      params.resource as keyof typeof resourceIdentifier
    ].model.findOrFail(params.resourceId)

    await mail.send((message) => {
      message
        .to(user.email)
        .subject(
          `Vous avez été à rejoindre ${resourceIdentifier[params.resource as keyof typeof resourceIdentifier].label}`
        )
        .htmlView('emails/invited_to_join_resource', {
          url: `${env.get('API_URL')}/api/v1/${params.resource}/${params.resourceId}`,
          email: payload.email,
          resource_label:
            resourceIdentifier[params.resource as keyof typeof resourceIdentifier].label,
          resource_name: resource.name,
          user_fullname: user.firstname + ' ' + user.lastname,
        })
    })

    return response.created({ ok: 'ok' })
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
