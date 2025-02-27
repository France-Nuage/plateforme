import { HttpRouterService } from '@adonisjs/core/types'
import RolesController from '#controllers/v1/iam/roles_controller'
const IAMPoliciesController = () => import('#controllers/v1/iam/policy_controller')
const IAMMembersController = () => import('#controllers/v1/iam/member_controller')

export const IamRoutes = (router: HttpRouterService) =>
  router
    .group(() => {
      router.resource('/iam/policies', IAMPoliciesController)
      router.resource('/iam/members', IAMMembersController)
    })
    .prefix('api/v1/:resource/:resourceId')

export const RolesRoutes = (router: HttpRouterService) =>
  router
    .group(() => {
      router.resource('roles', RolesController)
    })
    .prefix('iam')
