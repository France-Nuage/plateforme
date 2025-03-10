import factory from '@adonisjs/lucid/factories'
import { PermissionId } from '@france-nuage/types'
import Permission from '#models/iam/permission'
import { ServiceId } from '#models/catalog/service'
import { TypeId } from '#models/iam/type'
import { VerbId } from '#models/iam/verb'

export const PermissionFactory = factory
  .define(Permission, ({ faker }) => {
    return {
      id: faker.helpers.arrayElement(Object.values(PermissionId)),
      serviceId: faker.helpers.arrayElement(Object.values(ServiceId)),
      typeId: faker.helpers.arrayElement(Object.values(TypeId)),
      verbId: faker.helpers.arrayElement(Object.values(VerbId)),
    }
  })
  .build()
