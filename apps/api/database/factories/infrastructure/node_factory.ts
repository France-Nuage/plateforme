import factory from '@adonisjs/lucid/factories'
import { InstanceFactory } from '#database/factories/infrastructure/instance_factory'
import Node from '#models/infrastructure/node'

export const NodeFactory = factory
  .define(Node, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      url: faker.internet.url(),
      token: faker.string.uuid(),
      clusterId: faker.string.uuid(),
    }
  })
  .relation('instances', () => InstanceFactory)
  .build()
