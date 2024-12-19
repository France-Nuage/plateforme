// import User from '#models/user'
import RequestQueryBuilder from '../../../utils/RequestQueryBuilder.js'
import Role from '#models/iam/role'

export default {
  get: async function (
    id: string,
    includes: Array<string>,
    // user?: User
  ) {
    return await new RequestQueryBuilder(Role.query())
      .withIncludes(includes)
      .applyWhere([['id', id]])
      .firstOrFail()
  },
  list: async function (
    includes: Array<string>,
    // user?: User
  ) {
    return await new RequestQueryBuilder(Role.query()).withIncludes(includes).apply()
  },
}
