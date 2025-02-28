import factory from '@adonisjs/lucid/factories'
import Organization from '#models/resource/organization'

export const OrganizationFactory = factory
  .define(Organization, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      name: faker.company.name(),
    }
  })
  .build()
