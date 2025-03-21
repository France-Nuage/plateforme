import BasePolicy from '#policies/base_policy'
import authorization from '#services/authorization'
import User from '#models/user'
import { AuthorizerResponse } from '@adonisjs/bouncer/types'
import { PermissionId } from '#models/iam/permission'

export default class IAMMemberPolicy extends BasePolicy {
  /**
   * Every logged-in user can list an organization
   */
  index(user: User): AuthorizerResponse {
    return authorization.check(
      [
        PermissionId.ResourceManagerOrganizationsGetIamPolicy,
        PermissionId.ResourceManagerProjectsGetIamPolicy,
        PermissionId.ResourceManagerFoldersGetIamPolicy,
      ],
      user,
      this.resources
    )
  }

  show(user: User): AuthorizerResponse {
    return authorization.check(
      [
        PermissionId.ResourceManagerOrganizationsGetIamPolicy,
        PermissionId.ResourceManagerProjectsGetIamPolicy,
        PermissionId.ResourceManagerFoldersGetIamPolicy,
      ],
      user,
      this.resources
    )
  }

  store(user: User): AuthorizerResponse {
    return authorization.check(
      [
        PermissionId.ResourceManagerOrganizationsSetIamPolicy,
        PermissionId.ResourceManagerProjectsSetIamPolicy,
        PermissionId.ResourceManagerFoldersSetIamPolicy,
      ],
      user,
      this.resources
    )
  }
}
