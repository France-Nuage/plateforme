import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  async up() {
    this.schema.withSchema('infrastructure').table('instances', (table) => {
      table
        .enum('status', ['PROVISIONING', 'STAGING', 'RUNNING', 'STOPPING', 'TERMINATED'])
        .comment(
          'See this documentation : https://cloud.google.com/compute/docs/instances/instance-life-cycle?hl=fr'
        )
    })
  }

  async down() {
    this.schema.withSchema('infrastructure').table('instances', (table) => {
      table.dropColumn('status')
    })
  }
}
