import ProjectsController from '#controllers/v1/resource/projects_controller'
import { HttpRouterService } from '@adonisjs/core/types'

export const ProjectRoutes = (router: HttpRouterService) => {
  router
    .group(() => {
      router.resource('projects', ProjectsController)
    })
    .prefix('api/v1/')
}
