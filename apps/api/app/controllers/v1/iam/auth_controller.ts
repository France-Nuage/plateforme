import type { HttpContext } from '@adonisjs/core/http'
import { createUserValidator, loginUserValidator } from '#validators/v1/iam/user'
import User from '#models/user'

export default class AuthController {
  async register({ request, response }: HttpContext) {
    const payload = await request.validateUsing(createUserValidator)
    const user = await User.create({ ...payload })

    const token = await User.accessTokens.create(user, ['*'])

    return response.created({ token, user })
  }

  async generateToken({ request, response }: HttpContext) {
    const { email, password } = await request.validateUsing(loginUserValidator)
    const user = await User.verifyCredentials(email, password)
    const token = await User.accessTokens.create(user, ['*'])

    return response.ok({
      token: token,
      ...user.serialize(),
    })
  }

  async login({ request, response }: HttpContext) {
    const { email, password } = await request.validateUsing(loginUserValidator)
    const user = await User.verifyCredentials(email, password)
    const token = await User.accessTokens.create(user, ['*'])

    return response.ok(token)
  }

  async logout({ auth, response }: HttpContext) {
    const user = auth.getUserOrFail()
    const token = auth.user?.currentAccessToken.identifier
    if (!token) {
      return response.badRequest({ message: 'Token not found' })
    }
    await User.accessTokens.delete(user, token)
    return response.ok({ message: 'Logged out' })
  }

  async me({ auth, response }: HttpContext) {
    try {
      const user = auth.getUserOrFail()
      return response.ok(user)
    } catch (error) {
      return response.unauthorized({ error: 'User not found' })
    }
  }

  async preload({ request, response, auth }: HttpContext) {
    try {
      const user = auth.getUserOrFail()
      return response.notImplemented({ user })
    } catch (error) {
      return response.unauthorized({ error: 'User not found' })
    }
  }
}
