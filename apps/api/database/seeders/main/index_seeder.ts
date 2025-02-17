import { BaseSeeder } from '@adonisjs/lucid/seeders'
import app from '@adonisjs/core/services/app'

export default class MainSeeder extends BaseSeeder {
  private async seed(Seeder: { default: typeof BaseSeeder }) {
    /**
     * Do not run when not in a environment specified in Seeder
     */
    if (
      !Seeder.default.environment ||
      (!Seeder.default.environment.includes('development') && app.inDev) ||
      (!Seeder.default.environment.includes('testing') && app.inTest) ||
      (!Seeder.default.environment.includes('production') && app.inProduction)
    ) {
      return
    }

    await new Seeder.default(this.client).run()
  }

  public async run() {
    // CATALOG schema
    await this.seed(await import('#database/seeders/catalog/services_seeder'))

    // LOCALISATION schema
    await this.seed(await import('#database/seeders/localisation/countries_seeder'))

    // MEMBER schema
    await this.seed(await import('#database/seeders/member/users_seeder'))

    // IAM schema
    await this.seed(await import('#database/seeders/iam/policy_seeder'))
    await this.seed(await import('#database/seeders/iam/roles_seeder'))
    await this.seed(await import('#database/seeders/iam/types_seeder'))
    await this.seed(await import('#database/seeders/iam/verbs_seeder'))
    await this.seed(await import('#database/seeders/iam/permissions_seeder'))
    await this.seed(await import('#database/seeders/iam/role_permission_seeder'))
    await this.seed(await import('#database/seeders/iam/binding_seeder'))

    // RESOURCE schema
    await this.seed(await import('#database/seeders/resources/organizations_seeder'))
    await this.seed(await import('#database/seeders/resources/folders_seeder'))
    await this.seed(await import('#database/seeders/resources/projects_seeder'))

    // INFRASTRUCTURE schema
    await this.seed(await import('#database/seeders/infrastructure/regions_seeder'))
    await this.seed(await import('#database/seeders/infrastructure/zones_seeder'))
    await this.seed(await import('#database/seeders/infrastructure/clusters_seeder'))
  }
}
