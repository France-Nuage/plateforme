import env from '#start/env'
import { defineConfig, transports } from '@adonisjs/mail'

const mailConfig = defineConfig({
  default: 'smtp',

  from: {
    address: 'contact@france-nuage.fr',
    name: 'France Nuage',
  },

  replyTo: {
    address: 'no-reply@france-nuage.fr',
    name: 'France Nuage No Reply',
  },

  /**
   * The mailers object can be used to configure multiple mailers
   * each using a different transport or same transport with different
   * options.
   */
  mailers: {
    smtp: transports.smtp({
      host: env.get('SMTP_HOST', 'smtp-relay.brevo.com'),
      port: env.get('SMTP_PORT', 587),
      auth: {
        type: 'login',
        user: env.get('SMTP_USER', ''),
        pass: env.get('SMTP_PASSWORD', ''),
      },
    }),
  },
})

export default mailConfig

declare module '@adonisjs/mail/types' {
  export interface MailersList extends InferMailers<typeof mailConfig> {}
}
