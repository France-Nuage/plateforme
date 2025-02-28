import { test } from '@japa/runner'
import Region, { RegionName } from '#models/infrastructure/region'

test.group('Region', () => {
  test('every variant of the RegionName enum exists in the database', async ({ assert }) => {
    const regions = await Region.query().select('name')
    const regionNames = regions.map((region) => region.name)

    for (const regionName of Object.values(RegionName)) {
      assert.include(regionNames, regionName, `Region ${regionName} missing in the database`)
    }
  })

  test('every region in the database exists in the RegionName enum', async ({ assert }) => {
    const regions = await Region.query().select('name')
    const regionNames = regions.map((region) => region.name)

    for (const regionName of regionNames) {
      assert.include(
        Object.values(RegionName),
        regionName,
        `Region ${regionName} missing in the RegionId enum`
      )
    }
  })
})
