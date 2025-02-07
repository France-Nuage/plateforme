import BasePolicy from '#policies/base_policy'
import User from '#models/user'
import { AuthorizerResponse } from '@adonisjs/bouncer/types'

export default class MemberPolicy extends BasePolicy {
  /**
   * Only the folder creator can update the folder
   */
  update(user: User, entityUser: User): AuthorizerResponse {
    return user.id === entityUser.id
  }
}
