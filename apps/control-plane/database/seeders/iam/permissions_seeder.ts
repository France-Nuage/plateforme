import { BaseSeeder } from '@adonisjs/lucid/seeders'
import { PermissionId } from '@france-nuage/types'
import Permission from '#models/iam/permission'
import { VerbId } from '#models/iam/verb'
import { TypeId } from '#models/iam/type'
import { ServiceId } from '#models/catalog/service'

export default class extends BaseSeeder {
  static environment = ['development', 'production', 'testing']

  public async run() {
    await Permission.updateOrCreateMany(
      'id',
      Object.values(PermissionId).map((permission) => {
        const permissionSplit = permission.split('.')
        return {
          id: permission,
          serviceId: permissionSplit[0] as ServiceId,
          typeId: permissionSplit[1] as TypeId,
          verbId: permissionSplit[2] as VerbId,
        }
      })
    )
  }
}
