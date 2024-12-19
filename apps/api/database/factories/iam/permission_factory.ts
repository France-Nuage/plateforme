import factory from '@adonisjs/lucid/factories'
import Permission from '#models/iam/permission'

export const PermissionFactory = factory
  .define(Permission, ({ faker }) => {
    return {
      serviceId: faker.string.uuid(),
      typeId: faker.string.uuid(),
      verbId: faker.string.uuid(),
    }
  })
  .build()
