export default {
  allowedRedirects: (process.env.ALLOWED_REDIRECTS ?? `${process.env.CONSOLE_URL ?? 'https://console'}/auth/redirect/mock`).split(' '),
  clientId: process.env.OIDC_CLIENT_ID ?? 'francenuage',
  clientSecret: process.env.OIDC_CLIENT_SECRET ?? 'francenuage',
  host: process.env.HOST ?? 'oidc',
  port: process.env.PORT ?? 80,
  issuer: 'https://oidc',
}
