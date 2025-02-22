import factory from '@adonisjs/lucid/factories'
import Verb, { VerbId } from '#models/iam/verb'

export const VerbFactory = factory
  .define(Verb, ({ faker }) => {
    return {
      id: faker.helpers.arrayElement(Object.values(VerbId)),
    }
  })
  .build()
