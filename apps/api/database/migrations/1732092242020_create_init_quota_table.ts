import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  async up() {
    this.schema.createSchema('quota')
    this.schema.withSchema('quota').createTable('quota', (table) => {
      table.string('quota__id', 63)
      table
        .boolean('is_precise')
        .comment('Indique si le quota est suivi avec précision (TRUE) ou non (FALSE).')
      table
        .string('refresh_interval')
        .comment('Fréquence de réinitialisation du quota, ex: "minute", "day", "10 seconds".')
      table
        .string('metric_display_name')
        .comment('Nom lisible pour la métrique du quota, ex: "CPU Allocation".')
      table
        .string('quota_display_name')
        .comment('Nom lisible pour le quota, ex: "CPU Quota for Region par Zone par Project"')
      table.string('metric_unit').comment('Unité utilisée pour la métrique, ex: "cores", "GB".')
      table
        .boolean('is_fixed')
        .comment('Indique si la valeur du quota est fixe (TRUE) ou ajustable (FALSE).')
      table
        .string('service_request_quota_uri')
        .comment('URL permettant de demander plus de quota pour ce service.')
    })

    this.schema.withSchema('quota').createTable('metrics', (table) => {
      table.string('quota__id', 63)
      table.uuid('organization__id')
      table.uuid('folder__id')
      table.uuid('project__id')
      table.string('metric__id')
      table.string('metric__id')
      table
        .boolean('is_eligible')
        .comment('Indique si le quota est éligible pour une augmentation (TRUE/FALSE).')
      table
        .integer('ineligibility_reason')
        .comment(
          "Raison d'inéligibilité : 0=UNSPECIFIED, 1=NO_VALID_BILLING_ACCOUNT, 2=OTHER, etc."
        )

      table
        .foreign('organization__id')
        .references('organization__id')
        .inTable('resource.organizations')
        .onDelete('cascade')
        .onUpdate('restrict')
      table
        .foreign('folder__id')
        .references('folder__id')
        .inTable('resource.folders')
        .onDelete('cascade')
        .onUpdate('restrict')
      table
        .foreign('project__id')
        .references('project__id')
        .inTable('resource.projects')
        .onDelete('cascade')
        .onUpdate('restrict')

      table.unique(['organization__id', 'folder__id', 'project__id'])
    })
  }

  async down() {}
}
