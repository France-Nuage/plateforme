import env from '#start/env'

/**
 * The dev cluster is a ready-to-use proxmox cluster with some preconfigured basics,
 * used in development environment for ease-of-use.
 */
export const cluster = {
  id: env.get('DEV_CLUSTER_ID', '00000000-0000-0000-0000-000000000000'),
  name: env.get('DEV_CLUSTER_NAME', 'dev-cluster'),
  host: env.get('DEV_CLUSTER_HOST'),
  token: {
    id: env.get('DEV_CLUSTER_TOKEN_ID'),
    secret: env.get('DEV_CLUSTER_TOKEN_SECRET'),
  },
}

export const user = {
  email: env.get('DEV_USER_EMAIL', 'admin@france-nuage.fr'),
  password: env.get('DEV_USER_PASSWORD', 'password'),
}
