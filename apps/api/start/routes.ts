/*
|--------------------------------------------------------------------------
| Routes file
|--------------------------------------------------------------------------
|
| The routes file is used for defining the HTTP routes.
|
*/

import router from '@adonisjs/core/services/router'
import {middleware} from '#start/kernel'
import AutoSwagger from 'adonis-autoswagger'
import swagger from 'Config/swagger'
// import transmit from '@adonisjs/transmit/services/main'

// transmit.registerRoutes((route) => {
//   // Ensure you are authenticated to register your client
//   // route.middleware(middleware.auth())
//   // Add a throttle middleware to other transmit routes
//   // route.use(throttle)
// })

const ServicesController = () => import('#controllers/v1/services/services_controller') //controller not found I had to create a new controller
const OrganizationsController = () => import('#controllers/v1/resource/organizations_controller')
const ProjectsController = () => import('#controllers/v1/resource/projects_controller')
const FoldersController = () => import('#controllers/v1/resource/folders_controller')
const AuthController = () => import('#controllers/v1/iam/auth_controller')
const InstancesController = () => import('#controllers/v1/infrastructure/instances_controller')
const IAMPoliciesController = () => import('#controllers/v1/iam/policy_controller')
const IAMMembersController = () => import('#controllers/v1/iam/member_controller')
const BillingAccountController = () => import('#controllers/v1/billing/billing_accounts_controller')
const MembersController = () => import('#controllers/v1/member/members_controller')
const ZonesController = () => import('#controllers/v1/infrastructure/zones_controller')
const RegionsController = () => import('#controllers/v1/infrastructure/regions_controller')
const PricingController = () => import('#controllers/v1/billing/price_controller')
const PaymentMethodController = () => import('#controllers/v1/payment/payment_methods_controller')
const MetricsController = () => import('#controllers/v1/infrastructure/metrics_controller')

Route.get('/swagger', async () => {
  return AutoSwagger.docs(Route.toJSON(), swagger)
})

// Renders Swagger-UI and passes YAML-output of /swagger
Route.get('/docs', async () => {
  return AutoSwagger.ui('/swagger', swagger)
})
router
  .group(() => {
    router
      .group(() => {
        router.resource('folders', FoldersController)
        router.resource('organizations', OrganizationsController)
        router.resource('projects', ProjectsController)
        router.resource('services', ServicesController)
        router.resource('members', MembersController)
        router.resource('regions', RegionsController)
        router.resource('zones', ZonesController)
        router.resource('pricing', PricingController)
        router.resource('payment-methods', PaymentMethodController)
        router.resource('infrastructures', MetricsController) // Fixing spelling error ('resource' -> 'resource')

        router
          .group(() => {
            router.resource('accounts', BillingAccountController)
          })
          .prefix('billing')

        router
          .group(() => {
            router.resource('instances', InstancesController)
            router.post('price', [InstancesController, 'getPrice'])
          })
          .prefix('compute')
        router
          .group(() => {
            router.resource('/iam/policies', IAMPoliciesController)
            router.resource('/iam/members', IAMMembersController)
          })
          .prefix('/:resource/:resourceId')

        router.group(() => {
          router.resource('/folders/:folderId', BillingAccountController)
        })

        router.get('/auth/me', [AuthController, 'me'])
      })
      .middleware([middleware.auth()])

    router.group(() => {
      router.get('/infrastructure/:metrics', [MetricsController, 'store'])
    })

    router.post('/auth/register', [AuthController, 'register'])
    router.post('/auth/login', [AuthController, 'login'])
    router.post('/auth/token', [AuthController, 'generateToken'])
    router.post('/auth/reset-password-request', [AuthController, 'resetPasswordRequest'])
    router.get('/auth/reset-password-token/:token', [AuthController, 'resetPasswordToken'])
    router.post('/auth/reset-password', [AuthController, 'resetPassword'])
  })
  .prefix('api/v1')
