import factory from '@adonisjs/lucid/factories'
import Instance from '#models/infrastructure/instance'

export const InstanceFactory = factory
  .define(Instance, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      // name: faker.internet.userName(),
      // node: faker.internet.ipv4(),
      clusterId: faker.string.uuid(),
    }
  })
  .build()
