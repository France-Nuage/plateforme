import { test } from '@japa/runner'
import Zone, { ZoneName } from '#models/infrastructure/zone'

test.group('Zone', () => {
  test('every variant of the ZoneName enum exists in the database', async ({ assert }) => {
    const zones = await Zone.query().select('name')
    const zoneNames = zones.map((zone) => zone.name)

    for (const zoneName of Object.values(ZoneName)) {
      assert.include(zoneNames, zoneName, `Zone ${zoneName} missing in the database`)
    }
  })

  test('every zone in the database exists in the ZoneName enum', async ({ assert }) => {
    const zones = await Zone.query().select('name')
    const zoneNames = zones.map((zone) => zone.name)

    for (const zoneName of zoneNames) {
      assert.include(
        Object.values(ZoneName),
        zoneName,
        `Zone ${zoneName} missing in the ZoneId enum`
      )
    }
  })
})
