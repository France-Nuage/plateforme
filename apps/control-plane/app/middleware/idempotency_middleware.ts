import type { HttpContext } from '@adonisjs/core/http'
import type { NextFn } from '@adonisjs/core/types/http'
import redis from '@adonisjs/redis/services/main'
import { randomUUID } from 'node:crypto'

export default class IdempotencyMiddleware {
  async handle(ctx: HttpContext, next: NextFn) {
    if (['POST', 'PUT', 'PATCH'].includes(ctx.request.method())) {
      const idempotencyKey = ctx.request.header('x-idempotency-key')

      if (!idempotencyKey) {
        const uuid = randomUUID()
        await redis.set(uuid, '')
        await redis.expire(uuid, 60 * 10)
        ctx.response.safeHeader('x-idempotence-key', uuid)
        return await next()
      }

      if (!this.isValidUUID(idempotencyKey)) {
        return ctx.response.badRequest({
          message: 'Idempotency is invalid',
        })
      }

      const idempotenceKeyRedis = await redis.get(idempotencyKey)
      if (!idempotenceKeyRedis) {
        await redis.set(idempotencyKey, 'default value')
        await redis.expire(idempotencyKey, 60 * 10)
        ctx.response.safeHeader('x-idempotence-key', idempotencyKey)
        return await next()
      }

      return ctx.response.accepted({
        message: 'Request are already processed',
      })
    }

    /**
     * Call next method in the pipeline and return its output
     */
    const output = await next()
    return output
  }

  private isValidUUID(uuid: string) {
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i
    return uuidRegex.test(uuid)
  }
}
