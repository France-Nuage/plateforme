import { HttpRouterService } from '@adonisjs/core/types'
const InstancesController = () => import('#controllers/v1/infrastructure/instances_controller')

export const ComputeRoutes = (router: HttpRouterService) =>
  router
    .group(() => {
      router.post('instances/:id/start', [InstancesController, 'start'])
      router.post('instances/:id/stop', [InstancesController, 'stop'])
      router.resource('instances', InstancesController)
      router.post('price', [InstancesController, 'getPrice'])
    })
    .prefix('api/v1/compute')
