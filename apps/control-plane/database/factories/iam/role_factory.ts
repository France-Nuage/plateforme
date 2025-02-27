import factory from '@adonisjs/lucid/factories'
import Role from '#models/iam/role'

export const RoleFactory = factory
  .define(Role, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      serviceId: faker.string.uuid(),
    }
  })
  .build()
