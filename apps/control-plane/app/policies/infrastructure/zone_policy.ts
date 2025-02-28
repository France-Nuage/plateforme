import BasePolicy from '#policies/base_policy'
import authorization from '#services/authorization'
import User from '#models/user'
import { AuthorizerResponse } from '@adonisjs/bouncer/types'
import { PermissionId } from '#models/iam/permission'

export default class ZonePolicy extends BasePolicy {
  /**
   * Every logged-in user can list an organization
   */
  index(user: User): AuthorizerResponse {
    return authorization.check([PermissionId.ComputeZonesList], user, this.resources)
  }

  show(user: User): AuthorizerResponse {
    return authorization.check([PermissionId.ComputeZonesGet], user, this.resources)
  }
}
