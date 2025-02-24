import { HttpRouterService } from '@adonisjs/core/types'
import AuthController from '#controllers/v1/iam/auth_controller'
import { middleware } from '#start/kernel'

export const AuthRoutes = (router: HttpRouterService) =>
  router
    .group(() => {
      router.post('/register', [AuthController, 'register'])
      router.post('/login', [AuthController, 'login'])
      router.post('/token', [AuthController, 'generateToken'])
      router.post('/reset-password-request', [AuthController, 'resetPasswordRequest'])
      router.get('/reset-password-token/:token', [AuthController, 'resetPasswordToken'])
      router.post('/reset-password', [AuthController, 'resetPassword'])
      router.get('/me', [AuthController, 'me']).middleware([middleware.auth()])
    })
    .prefix('api/v1/auth')
