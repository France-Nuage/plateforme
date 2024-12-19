import factory from '@adonisjs/lucid/factories'
import Folder from '#models/resource/folder'

export const FolderFactory = factory
  .define(Folder, ({ faker }) => {
    return {
      id: faker.string.uuid(),
      name: faker.company.catchPhrase(),
      description: faker.lorem.sentence(),
    }
  })
  .build()
