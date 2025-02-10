import { test } from '@japa/runner'
import Type, { TypeId } from '#models/iam/type'

test.group('Type', () => {
  test('every variant of the TypeId enum exists in the database', async ({ assert }) => {
    const types = await Type.query().select('type__id')
    const typeIds = types.map((type) => type.id)

    for (const typeId of Object.values(TypeId)) {
      assert.include(typeIds, typeId, `Type ${typeId} missing in the database`)
    }
  })

  test('every type in the database exists in the TypeId enum', async ({ assert }) => {
    const types = await Type.query().select('type__id')
    const typeIds = types.map((type) => type.id)

    for (const typeId of typeIds) {
      assert.include(Object.values(TypeId), typeId, `Type ${typeId} missing in the VerbId enum`)
    }
  })
})
