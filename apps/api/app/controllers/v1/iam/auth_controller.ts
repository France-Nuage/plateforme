import type { HttpContext } from '@adonisjs/core/http'
import {
  createUserValidator,
  loginUserValidator,
  resetPasswordRequestValidator,
  resetPasswordValidator,
} from '#validators/v1/iam/user'
import User from '#models/user'
import { randomBytes } from 'node:crypto'
import { DateTime } from 'luxon'
import Token from '#models/iam/token'
import Env from '#start/env'
import mail from '@adonisjs/mail/services/main'
import Organization from '#models/resource/organization'
import Policy from '#models/iam/policy'
import Binding from '#models/iam/binding'
import Role from '#models/iam/role'
import db from '@adonisjs/lucid/services/db'

export default class AuthController {
  async register({ request, response }: HttpContext) {
    const trx = await db.transaction()
    const payload = await request.validateUsing(createUserValidator)

    try {
      const user = await User.create({ ...payload }, { client: trx })
      const organization = await Organization.create(
        { name: `${user.lastname}-organization` },
        { client: trx }
      )
      const policy = await Policy.create({ organizationId: organization.id }, { client: trx })
      const role = await Role.query({ client: trx })
        .where('role__id', 'roles/resourcemanager.organizationAdmin')
        .firstOrFail()
      await Binding.create(
        { memberId: user.id, policyId: policy.id, roleId: role.id },
        { client: trx }
      )

      await trx.commit()
      const token = await User.accessTokens.create(user, ['*'])

      return response.created({ token, user })
    } catch (error) {
      await trx.rollback()

      return response.badRequest({
        message: error.message,
      })
    }
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

  async resetPasswordRequest({ request, response }: HttpContext) {
    const { email } = await request.validateUsing(resetPasswordRequestValidator)

    const user = await User.findBy('email', email)
    if (!user) {
      return response.ok({ message: 'User not found' })
    }

    const token = randomBytes(32).toString('hex')
    const expiresAt = DateTime.now().plus({ minutes: 15 })

    await Token.create({
      email: user.email,
      token: token,
      expiresAt: expiresAt,
    })

    const resetUrl = `${Env.get('PLATFORM_URL')}/auth/reset-password?token=${token}`

    await mail.send((message) => {
      message
        .to(user.email)
        .subject('Réinitialisation de votre mot de passe')
        .htmlView('emails/reset_password', { url: resetUrl, email: email })
    })

    return response.ok({ message: 'Un lien de réinitialisation vous a été envoyé par e-mail.' })
  }

  async resetPasswordToken({ params, response }: HttpContext) {
    const token = await Token.findBy('token', params.token)

    if (!token) {
      return response.badRequest('Lien invalide')
    }

    if (DateTime.now() > token.expiresAt) {
      return response.badRequest('Le lien a expiré')
    }

    return response.ok({
      token,
    })
  }

  async resetPassword({ request, response }: HttpContext) {
    const { token, password } = await request.validateUsing(resetPasswordValidator)

    const tokenFromDb = await Token.findBy('token', token)
    if (!tokenFromDb) {
      return response.badRequest({ message: 'Invalid link' })
    }

    if (DateTime.now() > tokenFromDb.expiresAt) {
      await tokenFromDb.delete()
      return response.badRequest({ message: 'The link is expired' })
    }

    const user = await User.findByOrFail('email', tokenFromDb.email)

    user.password = password
    await user.save()

    await tokenFromDb.delete()

    return { message: 'Password reset successfully!' }
  }
}
