import RequestQueryBuilder from '../../../utils/RequestQueryBuilder.js'
import Zone from '#models/infrastructure/zone'

export default {
  get: async function (id: string, includes: Array<string>) {
    return new RequestQueryBuilder(Zone.query())
      .withIncludes(includes)
      .applyWhere([['id', '=', id]])
      .firstOrFail()
  },
  list: async function (
    includes: Array<string>
    // user?: User
  ) {
    return new RequestQueryBuilder(Zone.query())
      .withIncludes(includes)
      .withPagination(1, 10)
      .apply()
  },
  // create: async function (body: { [_: string]: string | number | null }, user: User) {},
  // update: async function (id, body) {},
  // delete: async function (id) {},
}
