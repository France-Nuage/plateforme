import ZonesController from '#controllers/v1/infrastructure/zones_controller'
import { HttpRouterService } from '@adonisjs/core/types'

export const ZoneRoutes = (router: HttpRouterService) => {
  router
    .group(() => {
      router.resource('zones', ZonesController)
    })
    .prefix('api/v1/')
}
