import { BaseModel, column } from '@adonisjs/lucid/orm'

export default class Verb extends BaseModel {
  public static table = 'iam.verbs'

  @column({ isPrimary: true, columnName: 'verb__id' })
  declare id: VerbId
}

export enum VerbId {
  Create = 'create',
  Get = 'get',
  GetIamPolicy = 'getIamPolicy',
  List = 'list',
  ListIamRoles = 'listIamRoles',
  SetIamPolicy = 'setIamPolicy',
  Update = 'update',
}
