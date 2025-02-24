import RegionsController from '#controllers/v1/infrastructure/regions_controller'
import { HttpRouterService } from '@adonisjs/core/types'

export const RegionRoutes = (router: HttpRouterService) => {
  router
    .group(() => {
      router.resource('regions', RegionsController)
    })
    .prefix('api/v1/')
}
