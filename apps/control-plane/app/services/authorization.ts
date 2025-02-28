import User from '#models/user'
import Binding from '#models/iam/binding'
import Role from '#models/iam/role'
import Policy from '#models/iam/policy'

const filterPolicyKey = {
  organization: 'organizationId',
  folder: 'folderId',
  project: 'projectId',
}

export default {
  check: async function (
    permissions: Array<String>,
    user: User,
    resources: Array<{ type: 'organization' | 'folder' | 'project'; id: string }> | [] = []
  ) {
    const bindingQuery = Binding.query()
      .where('memberId', user.id)
      .preload('role', (roleQuery) => {
        roleQuery.preload('permissions')
      })

    if (resources && resources.length > 0) {
      bindingQuery.preload('policy', (policyQuery) => {
        resources.forEach((resource) => {
          policyQuery.orWhere(filterPolicyKey[resource.type], resource.id)
        })
      })
    }

    const bindings = await bindingQuery

    const permissionsBinding = bindings
      .map((_) => _.role.permissions)
      .flat()
      .map((_) => {
        return `${_.serviceId}.${_.typeId}.${_.verbId}`
      })

    return permissionsBinding.some((r) => permissions.includes(r))
  },
  assign: async function (config: {
    user: User
    role: Role
    resource: { type: 'organization' | 'folder' | 'project'; id: string }
    policy: Policy
  }) {
    const policy = config.policy
      ? config.policy
      : await Policy.query()
          .where(filterPolicyKey[config.resource.type], config.resource.id)
          .firstOrFail()

    return await Binding.create({
      roleId: config.role.id,
      memberId: config.user.id,
      policyId: policy.id,
    })
  },
}
