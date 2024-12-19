import RequestQueryBuilder from '../../../utils/RequestQueryBuilder.js'
// import User from '#models/user'
import Account from '#models/billing/billing_account'

export default {
  get: async function (
    id: string,
    includes: Array<string>
    // user?: User
  ) {
    return new RequestQueryBuilder(Account.query())
      .withIncludes(includes)
      .applyWhere([['id', '=', id]])
      .firstOrFail()
  },
  list: async function (
    includes: Array<string>
    // user?: User
  ) {
    return new RequestQueryBuilder(Account.query())
      .withIncludes(includes)
      .withPagination(1, 10)
      .apply()
  },
  create: async function (
    body: { [_: string]: string | number | null }
    // user?: User
  ) {
    return await Account.create(body)
  },
  // update: async function (id, body, user: User) {},
  // delete: async function (id, user: User) {},
}
