import env from '#start/env'
import app from '@adonisjs/core/services/app'
import { Secret } from '@adonisjs/core/helpers'
import { defineConfig } from '@adonisjs/core/http'

/**
 * The app key is used for encrypting cookies, generating signed URLs,
 * and by the "encryption" module.
 *
 * The encryption module will fail to decrypt data if the key is lost or
 * changed. Therefore it is recommended to keep the app key secure.
 */
export const appKey = new Secret(env.get('APP_KEY'))

/**
 * The application environment.
 */
export const environment = env.get('NODE_ENV', 'development')

/**
 * The configuration settings used by the HTTP server
 */
export const http = defineConfig({
  generateRequestId: true,
  allowMethodSpoofing: false,

  /**
   * Enabling async local storage will let you access HTTP context
   * from anywhere inside your application.
   */
  useAsyncLocalStorage: false,

  /**
   * Manage cookies configuration. The settings for the session id cookie are
   * defined inside the "config/session.ts" file.
   */
  cookie: {
    domain: '',
    path: '/',
    maxAge: '2h',
    httpOnly: true,
    secure: app.inProduction,
    sameSite: 'lax',
  },
})

export const worker = {
  email: env.get('WORKER_USER_EMAIL', 'worker@france-nuage.fr'),
}

export const organizations = {
  franceNuage: {
    policyId: env.get('FRANCE_NUAGE_POLICY_ID', '6653a9f0-b451-456b-a278-cb03ae8beb89'),
    organizationId: env.get('FRANCE_NUAGE_ORGANIZATION_ID', '00000000-0000-0000-0000-000000000000'),
  },
}

export const rootOrganization = {
  id: env.get('ROOT_ORGANIZATION_ORGANIZATION_ID', '00000000-0000-0000-0000-000000000000'),
  name: env.get('ROOT_ORGANIZATION_ORGANIZATION_NAME', 'France Nuage'),

  policy: {
    id: env.get('ROOT_ORGANIZATION_POLICY_ID', '00000000-0000-0000-0000-000000000000'),
  },
}

export const defaultFolder = {
  name: 'Interne',
}

export const defaultProject = {
  name: 'Interne',
}
