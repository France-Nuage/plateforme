import factory from '@adonisjs/lucid/factories'
import Service from '#models/service/service'

export const ServiceFactory = factory
  .define(Service, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      name: faker.company.name(),
    }
  })
  .build()
