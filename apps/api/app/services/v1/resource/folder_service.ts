import RequestQueryBuilder from '../../../utils/RequestQueryBuilder.js'
import Folder from '#models/resource/folder'
import User from '#models/user'
// import Project from '#models/resource/project'
import Policy from '#models/iam/policy'
import Role from '#models/iam/role'
import Authorization from '#services/authorization'

export default {
  get: async function (id: string, includes: Array<string>) {
    return new RequestQueryBuilder(Folder.query())
      .withIncludes(includes)
      .applyWhere([['id', '=', id]])
      .firstOrFail()
  },
  list: async function (
    includes: Array<string>
    // user?: User
  ) {
    return new RequestQueryBuilder(Folder.query())
      .withIncludes(includes)
      .withPagination(1, 10)
      .apply()
  },
  create: async function (body: { [_: string]: string | number | null }, user: User) {
    const folder = await Folder.create(body)
    await Folder.create(body)
    const policy = await Policy.create({
      folderId: folder.id,
      organizationId: folder.organizationId,
    })

    const role = await Role.findOrFail('roles/resourcemanager.folderAdmin')
    await Authorization.assign({
      user,
      role,
      policy,
      resource: { type: 'folder', id: folder.id },
    })

    return folder
  },
  // update: async function (id, body) {},
  // delete: async function (id) {},
}
