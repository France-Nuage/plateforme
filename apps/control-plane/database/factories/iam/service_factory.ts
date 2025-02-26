import factory from '@adonisjs/lucid/factories'
import Service, { ServiceId } from '#models/catalog/service'

export const ServiceFactory = factory
  .define(Service, ({ faker }) => {
    return {
      service__id: faker.helpers.arrayElement(Object.values(ServiceId)),
      description: faker.lorem.sentence(),
    }
  })
  .build()
