import { HttpContext } from '@adonisjs/core/http'
import MemberPolicy from '#policies/member/member_policy'
import User from '#models/user'
import { updateUserValidator } from '#validators/v1/iam/user'

export default class MembersController {
  /**
   * Handle form submission for the edit action
   */
  async update({ params, request, response, bouncer }: HttpContext) {
    const user = await User.findOrFail(params.id)
    const payload = await request.validateUsing(updateUserValidator)
    await bouncer.with(MemberPolicy).authorize('update', user)

    user.lastname = payload.lastname || user.lastname
    user.firstname = payload.firstname || user.firstname

    await user.save()

    return response.created(user)
  }
}
