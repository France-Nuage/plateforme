import { HttpContext } from '@adonisjs/core/http'
import OSPolicy from '#policies/infrastructure/os_policy'

export default class OSController {
  async index({ response, request, bouncer }: HttpContext) {
    await bouncer.with(OSPolicy).authorize('index')

    return response.notImplemented({
      request: request,
    })
  }

  async show({ response, request, bouncer }: HttpContext) {
    await bouncer.with(OSPolicy).authorize('show')

    return response.notImplemented({
      request: request,
    })
  }
}
