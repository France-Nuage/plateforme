import BasePolicy from '#policies/BasePolicy'
import authorization from '#services/authorization'
import User from '#models/user'
import { AuthorizerResponse } from '@adonisjs/bouncer/types'

export default class IAMRolePolicy extends BasePolicy {
  /**
   * Every logged-in user can list an organization
   */
  index(user: User): AuthorizerResponse {
    return authorization.check(['cloudasset.assets.listIamRoles'], user, this.resources)
  }

  /**
   * Every logged-in user can list an organization
   */
  show(user: User): AuthorizerResponse {
    return authorization.check(['cloudasset.assets.listIamRoles'], user, this.resources)
  }
}
