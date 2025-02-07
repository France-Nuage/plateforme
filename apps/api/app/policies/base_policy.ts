import { BasePolicy as BouncerBasePolicy } from '@adonisjs/bouncer'
import { inject } from '@adonisjs/core'
import { HttpContext } from '@adonisjs/core/http'

@inject()
export default class BasePolicy extends BouncerBasePolicy {
  public resources: Array<{ type: 'organization' | 'folder' | 'project'; id: string }> = []

  constructor(protected ctx: HttpContext) {
    super()
    this.resources = ctx.resources
  }
}
