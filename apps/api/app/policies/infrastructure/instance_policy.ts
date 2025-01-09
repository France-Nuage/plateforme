import BasePolicy from '#policies/BasePolicy'
import authorization from '#services/authorization'
import User from '#models/user'
import { AuthorizerResponse } from '@adonisjs/bouncer/types'

export default class InstancePolicy extends BasePolicy {
  /**
   * Every logged-in user can list an organization
   */
  index(user: User): AuthorizerResponse {
    return authorization.check(['compute.images.list'], user, this.resources)
  }

  show(user: User): AuthorizerResponse {
    return authorization.check(['compute.images.get'], user, this.resources)
  }

  store(user: User): AuthorizerResponse {
    return authorization.check(['compute.images.get'], user, this.resources)
  }
}
