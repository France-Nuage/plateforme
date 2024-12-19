import BasePolicy from '#policies/BasePolicy'
import authorization from '#services/authorization'
import User from '#models/user'
import { AuthorizerResponse } from '@adonisjs/bouncer/types'

export default class IAMMemberPolicy extends BasePolicy {
  /**
   * Every logged-in user can list an organization
   */
  index(user: User): AuthorizerResponse {
    return authorization.check(
      [
        'resourcemanager.projects.getIamPolicy',
        'resourcemanager.organizations.getIamPolicy',
        'resourcemanager.folders.getIamPolicy',
      ],
      user,
      this.resources
    )
  }

  show(user: User): AuthorizerResponse {
    return authorization.check(
      [
        'resourcemanager.projects.getIamPolicy',
        'resourcemanager.organizations.getIamPolicy',
        'resourcemanager.folders.getIamPolicy',
      ],
      user,
      this.resources
    )
  }
}
