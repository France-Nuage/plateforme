import factory from '@adonisjs/lucid/factories'
import User from '#models/user'

export const UserFactory = factory
  .define(User, ({ faker }) => {
    return {
      lastname: faker.name.lastName(),
      firstname: faker.name.firstName(),
      email: faker.internet.email(),
      password: faker.internet.password(), // Vous pouvez hasher ce mot de passe si nécessaire
    }
  })
  .build()
