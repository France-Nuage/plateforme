import User from '#models/user'
import { AuthorizerResponse } from '@adonisjs/bouncer/types'
import authorization from '#services/authorization'
import BasePolicy from '#policies/base_policy'
import { PermissionId } from '#models/iam/permission'

export default class ProjectPolicy extends BasePolicy {
  /**
   * Every logged-in user can list an organization
   */
  index(user: User): AuthorizerResponse {
    return authorization.check([PermissionId.ResourceManagerProjectsList], user, this.resources)
  }

  /**
   * Every logged-in user can show a folder
   */
  show(user: User): AuthorizerResponse {
    return authorization.check([PermissionId.ResourceManagerProjectsGet], user, this.resources)
  }

  /**
   * Every logged-in user can store a folder
   */
  store(user: User): AuthorizerResponse {
    return authorization.check([PermissionId.ResourceManagerProjectsCreate], user, this.resources)
  }

  /**
   * Only the project creator can update the project
   */
  update(): AuthorizerResponse {
    return false
  }

  /**
   * Only the project creator can destroy the project
   */
  destroy(): AuthorizerResponse {
    return false
  }
}
