// eslint-disable-next-line @unicorn/filename-case
import swaggerJSDoc, { Options } from 'swagger-jsdoc'

const options: Options = {
  definition: {
    openapi: '3.0.0',
    info: {
      title: 'API Documentation',
      version: '1.0.0',
      description: "Documentation de l'API pour mon projet AdonisJS",
    },
    servers: [
      {
        url: 'http://localhost:3333', // Adresse de base de ton API
      },
    ],
  },
  apis: ['apps/api/start/routes.ts', './app/Controllers/Http/*.ts'], // Chemins vers les fichiers TypeScript contenant les annotations
}

const swaggerSpec = swaggerJSDoc(options)

export default swaggerSpec
