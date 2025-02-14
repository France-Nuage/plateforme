import env from '#start/env'

/**
 * The dev cluster is a ready-to-use proxmox cluster with some preconfigured basics,
 * used in development environment for ease-of-use.
 */
export const cluster = {
  id: env.get('DEV_CLUSTER_ID', '00000000-0000-0000-0000-000000000000'),
  name: env.get('DEV_CLUSTER_NAME', 'dev-cluster'),
}

/**
 * The dev node is a ready-to-use proxmox node attached to the dev cluster with some
 * preconfigured basics, used in development environment for ease-of-use.
 */
export const node = {
  id: env.get('DEV_NODE_ID', '00000000-0000-0000-0000-000000000000'),
  name: env.get('DEV_NODE_NAME', 'pve-node2'),
  token: env.get('DEV_NODE_TOKEN'),
  url: env.get('DEV_NODE_URL'),
}

/**
 * The cloudflare client id and secrets are used to authenticate against the cloudflare
 * API in any environment outside the France Nuage main network, where IP whitelisting
 * takes precedence over client authentication.
 */
export const cloudflare = {
  clientId: env.get('CLOUDFLARE_ACCESS_CLIENT_ID'),
  clientSecret: env.get('CLOUDFLARE_ACCESS_CLIENT_SECRET'),
}
