import type { HttpContext } from '@adonisjs/core/http'
import { createFolderValidator } from '#validators/v1/resource/folder'
import FolderService from '#services/v1/resource/folder_service'
import FolderPolicy from '#policies/resource/folder_policy'

export default class FoldersController {
  /**
   * Handle form submission for the create action
   */
  async store({ request, response, bouncer, auth }: HttpContext) {
    await bouncer.with(FolderPolicy).authorize('store')
    const user = await auth.getUserOrFail()
    const payload = await request.validateUsing(createFolderValidator)

    return response.created(await FolderService.create({ ...payload }, user))
  }

  /**
   * Show individual record
   */
  async show({ params, response, bouncer, request }: HttpContext) {
    await bouncer.with(FolderPolicy).authorize('show')
    const folder = await FolderService.get(params.id, request.qs().includes)

    if (!folder) {
      response.notFound(`Folder ${params.id} not found`)
    }

    return response.ok(folder)
  }
}
