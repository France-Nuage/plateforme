import User from '#models/user'
import { AuthorizerResponse } from '@adonisjs/bouncer/types'
import authorization from '#services/authorization'
import BasePolicy from '#policies/base_policy'
import { PermissionId } from '#models/iam/permission'

export default class OrganizationPolicy extends BasePolicy {
  /**
   * Every logged-in user can list an organization
   */
  index(user: User): AuthorizerResponse {
    return authorization.check([PermissionId.ResourceManagerOrganizationsList], user)
  }

  /**
   * Every logged-in user can show an organization
   */
  get(user: User): AuthorizerResponse {
    return authorization.check([PermissionId.ResourceManagerOrganizationsGet], user, this.resources)
  }

  /**
   * Every logged-in user can store an organization
   */
  store(): AuthorizerResponse {
    return false
  }

  /**
   * Only the organization creator can update the organization
   */
  update(): AuthorizerResponse {
    return false
  }

  /**
   * Only the organization creator can destroy the organization
   */
  destroy(): AuthorizerResponse {
    return false
  }
}
