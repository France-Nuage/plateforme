import MembersController from '#controllers/v1/member/members_controller'
import { HttpRouterService } from '@adonisjs/core/types'

export const MemberRoutes = (router: HttpRouterService) => {
  router
    .group(() => {
      router.resource('members', MembersController)
    })
    .prefix('api/v1/')
}
