import { BaseSeeder } from '@adonisjs/lucid/seeders'
import Verb, { VerbId } from '#models/iam/verb'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    await Verb.updateOrCreateMany(
      'id',
      Object.values(VerbId).map((verb) => ({
        id: verb,
      }))
    )
  }
}
