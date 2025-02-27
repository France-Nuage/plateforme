import factory from '@adonisjs/lucid/factories'
import Project from '#models/resource/project'

export const ProjectFactory = factory
  .define(Project, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      name: faker.company.catchPhrase(),
      description: faker.lorem.sentence(),
      organizationId: faker.string.uuid(),
      billingAccountId: faker.string.uuid(),
    }
  })
  .build()
