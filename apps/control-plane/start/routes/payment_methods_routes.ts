import PaymentMethodController from '#controllers/v1/payment/payment_methods_controller'
import { HttpRouterService } from '@adonisjs/core/types'

export const PaymentMethodRoutes = (router: HttpRouterService) => {
  router
    .group(() => {
      router.resource('payment-methods', PaymentMethodController)
    })
    .prefix('api/v1/')
}
