import BasePolicy from '#policies/base_policy'
import authorization from '#services/authorization'
import User from '#models/user'
import { AuthorizerResponse } from '@adonisjs/bouncer/types'

export default class InstancePolicy extends BasePolicy {
  /**
   * Every logged-in user can list an organization
   */
  index(user: User): AuthorizerResponse {
    return authorization.check(['compute.instances.list'], user, this.resources)
  }

  show(user: User): AuthorizerResponse {
    return authorization.check(['compute.instances.get'], user, this.resources)
  }

  store(user: User): AuthorizerResponse {
    return authorization.check(['compute.instances.create'], user, this.resources)
  }

  update(user: User): AuthorizerResponse {
    return authorization.check(['compute.instances.update'], user, this.resources)
  }
}
