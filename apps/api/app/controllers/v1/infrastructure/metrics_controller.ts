import type { HttpContext } from '@adonisjs/core/http'
import axios from 'axios'
import Env from '#start/env'
import snappy from 'snappy'
import protobuf from 'protobufjs'

export default class MetricsController {
  /**
   * Handle the POST request to receive and push metrics
   */
  async store({ request, response }: HttpContext) {
    // Receive metrics data from the external agent
    const data = request.only([
      'ip_address',
      'hostname',
      'total_memory',
      'cpu_count',
      'disk_space',
      'os',
      'os_version',
      'installed_packages',
    ])

    // Validate required fields
    if (
      !data.ip_address ||
      !data.hostname ||
      !data.total_memory ||
      !data.cpu_count ||
      !data.disk_space ||
      !data.os ||
      !data.os_version ||
      !Array.isArray(data.installed_packages)
    ) {
      return response.badRequest({ error: 'Invalid data received' })
    }

    if (
      typeof data.ip_address !== 'string' ||
      typeof data.hostname !== 'string' ||
      typeof data.total_memory !== 'number' ||
      typeof data.cpu_count !== 'number' ||
      typeof data.disk_space !== 'number' ||
      typeof data.os !== 'string' ||
      typeof data.os_version !== 'string'
    ) {
      return response.badRequest({ error: 'Invalid data received' })
    }

    // Format data for Mimir (Prometheus Remote Write format)
    const root = await protobuf.load('./app/controllers/v1/infrastructure/mimir.proto')
    const writeRequestType = root.lookupType('prometheus.WriteRequest')
    const writeRequest = writeRequestType.create({
      timeseries: [
        {
          labels: [
            { name: '__name__', value: 'my_app_requests_total' },
            { name: 'instance', value: 'test' },
          ],
          samples: [{ value: 42, timestamp: Date.now() }],
        },
      ],
    })

    //
    // Encode in binary (Protobuf)
    const messageBuffer = writeRequestType.encode(writeRequest).finish()

    try {
      const compressed = await new Promise<Buffer>((resolve, reject) => {
        snappy.compress(
          Buffer.from(messageBuffer),
          {},
          (err: Error | null, compressed?: Buffer) => {
            if (err) {
              reject(err)
            } else if (!compressed) {
              reject(new Error('Compression returned undefined buffer'))
            } else {
              resolve(compressed)
            }
          }
        )
      })

      try {
        // Push data to Mimir
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
          console.error('Error pushing metrics:', mimirResponse.data)
          return response.internalServerError({
            error: 'Failed to push metrics',
            details: mimirResponse.data,
          })
        }

        // Respond to the external agent
        return response.ok({
          message: 'Metrics received and pushed successfully',
          pushedData: compressed,
        })
      } catch (error) {
        console.error('Error pushing metrics:', error.message)
        return response.internalServerError({
          error: 'Failed to push metrics',
          details: error.message,
        })
      }
    } catch (error) {
      console.error('Error compressing data:', error)
      return response.internalServerError({
        error: 'Failed to compress data',
        details: error.message,
      })
    }
  }
}
