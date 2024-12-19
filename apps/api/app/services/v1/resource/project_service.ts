import RequestQueryBuilder from '../../../utils/RequestQueryBuilder.js'
// import User from '#models/user'
import Project from '#models/resource/project'
import User from '#models/user'
import Policy from '#models/iam/policy'
import Role from '#models/iam/role'
import Authorization from '#services/authorization'
import Folder from "#models/resource/folder";

export default {
  get: async function (id: string, includes: Array<string>) {
    return new RequestQueryBuilder(Project.query())
      .withIncludes(includes)
      .applyWhere([['id', '=', id]])
      .firstOrFail()
  },
  list: async function (
    includes: Array<string>
    // user?: User
  ) {
    return new RequestQueryBuilder(Project.query())
      .withIncludes(includes)
      .withPagination(1, 10)
      .apply()
  },
  create: async function (body: { [_: string]: string | number | null }, user: User) {
    const project = await Project.create(body)
    const folder = await Folder.findOrFail(project.folderId)
    const policy = await Policy.create({
      organizationId: folder.organizationId,
      folderId: project.folderId,
      projectId: project.id,
    })
    const role = await Role.findOrFail('roles/resourcemanager.projectAdmin')

    await Authorization.assign({
      user,
      role,
      policy,
      resource: { type: 'project', id: project.id },
    })

    return project
  },
  // update: async function (id, body) {},
  // delete: async function (id) {},
}
