import OrganizationsController from '#controllers/v1/resource/organizations_controller'
import { HttpRouterService } from '@adonisjs/core/types'

export const OrganizationRoutes = (router: HttpRouterService) => {
  router
    .group(() => {
      router.resource('organizations', OrganizationsController)
    })
    .prefix('api/v1/')
}
