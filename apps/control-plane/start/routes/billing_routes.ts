import BillingAccountController from '#controllers/v1/billing/billing_accounts_controller'
import { HttpRouterService } from '@adonisjs/core/types'

export const BillingRoute = (router: HttpRouterService) =>
  router
    .group(() => {
      router.resource('accounts', BillingAccountController)
      router.resource('billing/folders/:folderId', BillingAccountController)
    })
    .prefix('api/v1')
