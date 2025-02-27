import factory from '@adonisjs/lucid/factories'
import Type, { TypeId } from '#models/iam/type'

export const TypeFactory = factory
  .define(Type, ({ faker }) => {
    return {
      type__id: faker.helpers.arrayElement(Object.values(TypeId)),
      description: faker.lorem.sentence(),
    }
  })
  .build()
