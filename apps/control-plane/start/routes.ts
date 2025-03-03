/*
|--------------------------------------------------------------------------
| Routes file
|--------------------------------------------------------------------------
|
| The routes file is used for defining the HTTP routes.
|
*/

import router from '@adonisjs/core/services/router'
import { middleware } from '#start/kernel'
// import transmit from '@adonisjs/transmit/services/main'

// transmit.registerRoutes((route) => {
//   // Ensure you are authenticated to register your client
//   // route.middleware(middleware.auth())
//   // Add a throttle middleware to other transmit routes
//   // route.use(throttle)
// })

import { ComputeRoutes } from '#start/routes/compute_routes'
import { IamRoutes, RolesRoutes } from '#start/routes/iam_routes'
import { AuthRoutes } from '#start/routes/authentification_routes'
import { BillingRoute } from '#start/routes/billing_routes'
import { FolderRoutes } from '#start/routes/folder_routes'
import { OrganizationRoutes } from '#start/routes/organization_routes'
import { ProjectRoutes } from '#start/routes/project_routes'
import { PaymentMethodRoutes } from '#start/routes/payment_methods_routes'
import { ZoneRoutes } from '#start/routes/zone_routes'
import { MemberRoutes } from '#start/routes/member_routes'
import { RegionRoutes } from '#start/routes/region_routes'
import { QemuRoutes } from '#start/routes/qemu_routes'

AuthRoutes(router)
QemuRoutes(router)

router
  .group(() => {
    ComputeRoutes(router)
    IamRoutes(router)
    RolesRoutes(router)
    BillingRoute(router)
    OrganizationRoutes(router)
    FolderRoutes(router)
    ProjectRoutes(router)
    PaymentMethodRoutes(router)
    ZoneRoutes(router)
    MemberRoutes(router)
    RegionRoutes(router)
  })
  .middleware([middleware.auth()])

router.get('/health', ({ response }) => {
  response.ok({ status: 'ok' })
})
