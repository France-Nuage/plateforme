import BasePolicy from '#policies/base_policy'
import authorization from '#services/authorization'
import User from '#models/user'
import { AuthorizerResponse } from '@adonisjs/bouncer/types'
import { PermissionId } from '@france-nuage/types'

export default class InstancePolicy extends BasePolicy {
  /**
   * Every logged-in user can list an organization
   */
  index(user: User): AuthorizerResponse {
    return authorization.check([PermissionId.ComputeInstancesList], user, this.resources)
  }

  show(user: User): AuthorizerResponse {
    return authorization.check([PermissionId.ComputeInstancesGet], user, this.resources)
  }

  store(user: User): AuthorizerResponse {
    return authorization.check([PermissionId.ComputeInstancesCreate], user, this.resources)
  }

  update(user: User): AuthorizerResponse {
    return authorization.check([PermissionId.ComputeInstancesUpdate], user, this.resources)
  }
}
