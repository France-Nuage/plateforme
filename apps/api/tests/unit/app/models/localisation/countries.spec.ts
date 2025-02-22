import { test } from '@japa/runner'
import Country, { CountryCode } from '#models/localisation/country'

test.group('Country', () => {
  test('every variant of the CountryCode enum exists in the database', async ({ assert }) => {
    const countries = await Country.query().select('code')
    const countryCodes = countries.map((country) => country.code)

    for (const countryCode of Object.values(CountryCode)) {
      assert.include(countryCodes, countryCode, `Country ${countryCode} missing in the database`)
    }
  })

  test('every country in the database exists in the CountryCode enum', async ({ assert }) => {
    const countries = await Country.query().select('code')
    const countryCodes = countries.map((country) => country.code)

    for (const countryCode of countryCodes) {
      assert.include(
        Object.values(CountryCode),
        countryCode,
        `Country ${countryCode} missing in the CountryCode enum`
      )
    }
  })
})
