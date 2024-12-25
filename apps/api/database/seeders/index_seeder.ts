import { BaseSeeder } from '@adonisjs/lucid/seeders'
import app from '@adonisjs/core/services/app'

export default class IndexSeeder extends BaseSeeder {
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
    // IAM
    await this.seed(await import('#database/seeders/iam/permission_seeder'))

    // SERVICE
    await this.seed(await import('#database/seeders/service/service_seeder'))

    // INFRASTRUCTURE
    await this.seed(await import('#database/seeders/infrastructure/infrastructure_seeder'))

    // BILLING
    await this.seed(await import('#database/seeders/billing/price_seeder'))

    // QUOTA
  }
}
