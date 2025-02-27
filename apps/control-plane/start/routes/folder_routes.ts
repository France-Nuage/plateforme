import FoldersController from '#controllers/v1/resource/folders_controller'
import { HttpRouterService } from '@adonisjs/core/types'

export const FolderRoutes = (router: HttpRouterService) => {
  router
    .group(() => {
      router.resource('folders', FoldersController)
    })
    .prefix('api/v1/')
}
