import BasePolicy from '#policies/BasePolicy'
import User from '#models/user'
import {AuthorizerResponse} from '@adonisjs/bouncer/types'

export default class OSPolicy extends BasePolicy {
  /**
   * Every logged-in user can list an organization
   */
  index(user: User): AuthorizerResponse {
    return true // authorization.check(['compute.images.list'], user, this.resources)
  }

  show(user: User): AuthorizerResponse {
    return true // authorization.check(['compute.images.get'], user, this.resources)
  }
}
