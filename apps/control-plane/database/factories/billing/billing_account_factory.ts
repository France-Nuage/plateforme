import factory from '@adonisjs/lucid/factories'
import BillingAccount from '#models/billing/billing_account'

export const BillingAccountFactory = factory
  .define(BillingAccount, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      stripe_customer__id: faker.string.uuid(),
    }
  })
  .build()
