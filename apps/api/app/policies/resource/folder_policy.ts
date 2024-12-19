import User from '#models/user'
import { AuthorizerResponse } from '@adonisjs/bouncer/types'
import BasePolicy from '#policies/BasePolicy'
import authorization from '#services/authorization'

export default class FolderPolicy extends BasePolicy {
  /**
   * Every logged-in user can list an organization
   */
  index(user: User): AuthorizerResponse {
    return authorization.check(['resourcemanager.folders.list'], user, this.resources)
  }

  /**
   * Every logged-in user can show a folder
   */
  show(user: User): AuthorizerResponse {
    return authorization.check(['resourcemanager.folders.get'], user, this.resources)
  }

  /**
   * Every logged-in user can store a folder
   */
  store(user: User): AuthorizerResponse {
    return authorization.check(['resourcemanager.folders.create'], user, this.resources)
  }

  /**
   * Only the folder creator can update the folder
   */
  update(): AuthorizerResponse {
    return false
  }

  /**
   * Only the folder creator can destroy the folder
   */
  destroy(): AuthorizerResponse {
    return true
  }
}
