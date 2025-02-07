/*
|--------------------------------------------------------------------------
| Environment variables service
|--------------------------------------------------------------------------
|
| The `Env.create` method creates an instance of the Env service. The
| service validates the environment variables and also cast values
| to JavaScript data types.
|
*/

import { Env } from '@adonisjs/core/env'

export default await Env.create(new URL('../', import.meta.url), {
  NODE_ENV: Env.schema.enum(['development', 'production', 'test'] as const),
  PORT: Env.schema.number(),
  APP_KEY: Env.schema.string(),
  PLATFORM_URL: Env.schema.string(),
  HOST: Env.schema.string({ format: 'host' }),
  LOG_LEVEL: Env.schema.enum.optional(['fatal', 'error', 'warn', 'info', 'debug', 'trace']),
  API_URL: Env.schema.string(),

  /*
  |----------------------------------------------------------
  | Variables for configuring database connection
  |----------------------------------------------------------
  */
  DB_HOST: Env.schema.string({ format: 'host' }),
  DB_PORT: Env.schema.number(),
  DB_USER: Env.schema.string(),
  DB_PASSWORD: Env.schema.string.optional(),
  DB_DATABASE: Env.schema.string(),

  /*
  |----------------------------------------------------------
  | Variables for configuring the limiter package
  |----------------------------------------------------------
  */
  LIMITER_STORE: Env.schema.enum.optional(['redis', 'memory'] as const),

  /*
  |----------------------------------------------------------
  | Variables for configuring the mail package
  |----------------------------------------------------------
  */
  BREVO_API_KEY: Env.schema.string.optionalWhen(process.env.NODE_ENV !== 'production'),
  SMTP_HOST: Env.schema.string.optional(),
  SMTP_PORT: Env.schema.number.optional(),
  SMTP_USER: Env.schema.string.optionalWhen(process.env.NODE_ENV !== 'production'),
  SMTP_PASSWORD: Env.schema.string.optionalWhen(process.env.NODE_ENV !== 'production'),

  /*
  |----------------------------------------------------------
  | Variables for configuring the drive package
  |----------------------------------------------------------
  */
  DRIVE_DISK: Env.schema.enum.optional(['fs', 'r2'] as const),
  R2_KEY: Env.schema.string.optionalWhen(process.env.DRIVE_DISK !== 'r2'),
  R2_SECRET: Env.schema.string.optionalWhen(process.env.DRIVE_DISK !== 'r2'),
  R2_BUCKET: Env.schema.string.optionalWhen(process.env.DRIVE_DISK !== 'r2'),
  R2_ENDPOINT: Env.schema.string.optionalWhen(process.env.DRIVE_DISK !== 'r2'),

  /*
 |----------------------------------------------------------
 | Variables for configuring the payment package
 |----------------------------------------------------------
 */
  STRIPE_SECRET_KEY: Env.schema.string(),

  REDIS_HOST: Env.schema.string({ format: 'host' }),
  REDIS_PORT: Env.schema.number.optional(),
  REDIS_PASSWORD: Env.schema.string.optional(),

  /*
  |----------------------------------------------------------
  | Variables for configuring the Cloudflare authentication headers
  |----------------------------------------------------------
  */
  CLOUDFLARE_ACCESS_CLIENT_ID: Env.schema.string.optional(),
  CLOUDFLARE_ACCESS_CLIENT_SECRET: Env.schema.string.optional(),
})
