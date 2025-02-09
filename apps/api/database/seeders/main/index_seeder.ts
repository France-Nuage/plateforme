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
    await this.seed(await import('#database/seeders/catalog/services_seeder'))
    await this.seed(await import('#database/seeders/iam/roles_seeder'))
    await this.seed(await import('#database/seeders/iam/types_seeder'))
    await this.seed(await import('#database/seeders/iam/verbs_seeder'))
    await this.seed(await import('#database/seeders/iam/permissions_seeder'))
    await this.seed(await import('#database/seeders/iam/role_permission_seeder'))
  }
}
