import { test } from '@japa/runner'
import Verb, { VerbId } from '#models/iam/verb'

test.group('Verb', () => {
  test('every variant of the VerbId enum exists in the database', async ({ assert }) => {
    const verbs = await Verb.query().select('verb__id')
    const verbIds = verbs.map((verb) => verb.id)

    for (const verbId of Object.values(VerbId)) {
      assert.include(verbIds, verbId, `Verb ${verbId} missing in the database`)
    }
  })

  test('every verb in the database exists in the VerbId enum', async ({ assert }) => {
    const verbs = await Verb.query().select('verb__id')
    const verbIds = verbs.map((verb) => verb.id)

    for (const verbId of verbIds) {
      assert.include(Object.values(VerbId), verbId, `Verb ${verbId} missing in the VerbId enum`)
    }
  })
})
