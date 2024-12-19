import RequestQueryBuilder from '../../../utils/RequestQueryBuilder.js'
import Organization from '#models/resource/organization'
import User from '#models/user'
import Policy from '#models/iam/policy'
import Authorization from '#services/authorization'
import Role from '#models/iam/role'

export default {
  get: async function (
    id: string,
    includes: Array<string>,
    user: User
  ) {
    return new RequestQueryBuilder(Organization.query())
      .withIncludes(includes)
      .applyWhere([['id', '=', id]])
      .firstOrFail()
  },
  list: async function (includes: Array<string>, user: User) {
    await user.load('policies')
    return new RequestQueryBuilder(Organization.query())
      .withIncludes(includes)
      .applyWhere([['id', 'in', user.policies.map((policy: Policy) => policy.organizationId)]])
      .withPagination(1, 10)
      .apply()
  },
  create: async function (body: { [_: string]: string | number | null }, user: User) {
    const organization = await Organization.create(body)
    const policy = await Policy.create({ organizationId: organization.id })
    const role = await Role.findOrFail('roles/resourcemanager.organizationAdmin')

    await Authorization.assign({
      user,
      role,
      policy,
      resource: { type: 'organization', id: organization.id },
    })

    return organization
  },
  // update: async function (id, body, user: User) {},
  // delete: async function (id, user: User) {},
}
