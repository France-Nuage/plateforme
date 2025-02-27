import { defineConfig } from '@adonisjs/ally'

const allyConfig = defineConfig({})

export default allyConfig

declare module '@adonisjs/ally/types' {
  interface SocialProviders extends InferSocialProviders<typeof allyConfig> {}
}
