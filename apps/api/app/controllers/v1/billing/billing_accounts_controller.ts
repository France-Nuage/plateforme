import { HttpContext } from '@adonisjs/core/http'
import BillingPolicy from '#policies/billing/billing_policy'
import { createAccountValidator } from '#validators/v1/billing/account'
import billing_account_service from '#services/v1/billing/billing_account_service'

export default class BillingAccountController {
  async index({ response, bouncer, request }: HttpContext) {
    await bouncer.with(BillingPolicy).authorize('index')

    const accounts = await billing_account_service.list(request.qs().includes)

    return response.ok(accounts)
  }

  async show({ response, bouncer, params, request }: HttpContext) {
    await bouncer.with(BillingPolicy).authorize('show')

    const account = await billing_account_service.get(params.id, request.qs().includes)

    return response.ok(account)
  }

  async store({ response, request, bouncer }: HttpContext) {
    await bouncer.with(BillingPolicy).authorize('store')
    const payload = await request.validateUsing(createAccountValidator)

    const account = await billing_account_service.create(payload)

    return response.created(account)
  }

  async update({ response, request, bouncer, params }: HttpContext) {
    await bouncer.with(BillingPolicy).authorize('update')
    // const payload = await request.validateUsing(updateAccountValidator)

    return response.notImplemented({
      params: params,
      request: request,
    })
  }

  async destroy({ response, request, bouncer, params }: HttpContext) {
    await bouncer.with(BillingPolicy).authorize('destroy')

    return response.notImplemented({
      params: params,
      request: request,
    })
  }
}
