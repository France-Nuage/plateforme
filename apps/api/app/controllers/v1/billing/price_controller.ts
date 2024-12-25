import { HttpContext } from '@adonisjs/core/http'

export default class PriceController {
  async index({ request, response }: HttpContext) {
    return response.notImplemented({
      request,
      response,
    })
  }

  async show({}: HttpContext) {}
}
