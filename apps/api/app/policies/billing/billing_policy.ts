import BasePolicy from '#policies/BasePolicy'
import authorization from '#services/authorization'
import User from '#models/user'
import { AuthorizerResponse } from '@adonisjs/bouncer/types'

export default class BillingPolicy extends BasePolicy {
  index(user: User): AuthorizerResponse {
    return authorization.check([], user, this.resources)
  }

  show(user: User): AuthorizerResponse {
    return authorization.check([], user, this.resources)
  }

  store(user: User): AuthorizerResponse {
    return authorization.check([], user, this.resources)
  }

  update(user: User): AuthorizerResponse {
    return authorization.check([], user, this.resources)
  }

  destroy(user: User): AuthorizerResponse {
    return authorization.check([], user, this.resources)
  }
}
