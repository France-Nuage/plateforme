import type { HttpContext } from '@adonisjs/core/http'
import { MetricsValidator } from '#validators/v1/infrastructure/metric'
import axios from 'axios'
import Env from '#start/env'
import snappy from 'snappy'
import protobuf from 'protobufjs'
import mimir from '../../../../protocol/mimir.proto'

export default class MetricsController {
  /**
   * Handle the POST request to receive and push metrics
   */
  async store({ request, response }: HttpContext) {
    try {
      // Validate request data
      const data = await request.validateUsing(MetricsValidator)

      // Load and parse the Protobuf schema
      const root = await protobuf.load(mimir)
      const writeRequestType = root.lookupType('prometheus.WriteRequest')

      // Create the Protobuf structure based on validated data
      const writeRequest = writeRequestType.create({
        timeseries: [
          {
            labels: [
              { name: 'instance', value: data.hostname },
              { name: 'ip_address', value: data.ip_address },
              { name: 'os', value: data.os },
              { name: 'os_version', value: data.os_version },
            ],
            samples: [
              { value: data.cpu_count, timestamp: Date.now() },
              { value: data.total_memory, timestamp: Date.now() },
              { value: data.disk_space, timestamp: Date.now() },
            ],
          },
        ],
      })

      // Encode the data into binary format using Protobuf
      const messageBuffer = writeRequestType.encode(writeRequest).finish()

      // Compress using Snappy
      const compressed = await snappy.compress(Buffer.from(messageBuffer))

      // Push the metrics to the mimir service
      const mimirUrl = Env.get('MIMIR_URL') + '/api/v1/push'
      const mimirResponse = await axios.post(mimirUrl, compressed, {
        headers: {
          'CF-Access-Client-Id': Env.get('CLOUDFLARE_CLIENT_ID'),
          'CF-Access-Client-Secret': Env.get('CLOUDFLARE_CLIENT_SECRET'),
          'Content-Type': 'application/x-protobuf',
          'Content-Encoding': 'snappy',
          'X-Prometheus-Remote-Write-Version': '0.1.0',
        },
      })

      if (mimirResponse.status !== 200) {
        throw new Error('Failed to push metrics: ' + JSON.stringify(mimirResponse.data))
      }

      return response.ok({
        message: 'Metrics received and pushed successfully',
        receivedData: data,
        pushedData: compressed.toString('base64'),
      })
    } catch (error: any) {
      console.error('Error processing metrics:', error.message)
      return response.internalServerError({ error: error.message })
    }
  }
}
