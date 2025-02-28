import { HttpRouterService } from '@adonisjs/core/types'

const QemuController = () => import('#controllers/internal/qemu_controller')

export const QemuRoutes = (router: HttpRouterService) =>
  router
    .group(() => {
      router.get('hypervisor/nodes/:node_id/qemu/:qemu_id/status/current', [
        QemuController,
        'currentStatus',
      ])
    })
    .prefix('api/internal')
