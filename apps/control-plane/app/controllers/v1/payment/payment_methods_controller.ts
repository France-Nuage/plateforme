import { Stripe } from 'stripe'
import env from '#start/env'
import { HttpContext } from '@adonisjs/core/http'
import PaymentProfile from '#models/billing/payment_profile'

export default class PaymentsController {
  private stripe

  constructor() {
    this.stripe = new Stripe(env.get('STRIPE_SECRET_KEY'))
  }

  public async index({ customer_id }: { customer_id: string }): Promise<any> {
    return this.stripe.paymentMethods.list({
      customer: customer_id,
      type: 'card',
    })
  }

  /**
   * @param customer_id
   */
  public async store({ response }: HttpContext): Promise<any> {
    const customer = await this.stripe.customers.create()

    if (!customer) {
      response.notFound('An error occured')
    }

    try {
      await PaymentProfile.create({ stripeCustomerId: customer.id })
    } catch (e) {
      await this.stripe.customers.del(customer.id)
    }

    const setupIntent = await this.stripe.setupIntents.create({
      payment_method_types: ['card'],
      customer: customer.id,
    })

    return response.ok({ clientSecret: setupIntent.client_secret })
  }

  /**
   * @param id
   */
  public async show(id: string): Promise<Stripe.PaymentMethod> {
    return await this.stripe.paymentMethods.retrieve(id)
  }
}
