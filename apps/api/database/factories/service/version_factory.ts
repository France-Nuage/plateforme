import factory from '@adonisjs/lucid/factories'
import Version from '#models/service/version'

export const VersionFactory = factory
  .define(Version, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      name: faker.company.catchPhrase(),
      description: faker.lorem.sentence(),
      // availableAt: faker.date.recent(),
      serviceId: faker.string.uuid(),
    }
  })
  .build()
