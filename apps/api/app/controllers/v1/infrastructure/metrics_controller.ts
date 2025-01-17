import type { HttpContext } from '@adonisjs/core/http'
import axios from 'axios' // Import HTTP client
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
      !data.installed_packages
    ) {
      return response.badRequest({ error: 'Invalid data received' })
    }

    // Format data for Mimir (Prometheus Remote Write format)
    /* const formattedMetrics = await snappy.compress(
      Buffer.from(
        JSON.stringify({
          "timeseries": [
            {
              "labels": [
                {"name": "__name__", "value": "http_requests_total"},
                {"name": "method", "value": "GET"},
                {"name": "status", "value": "200"}
              ],
              "samples": [
                {"value": 42, "timestamp": 1673945600000}
              ]
            }
          ]
        })
      )
    )*/
    const root = await protobuf.load('plop.proto')
    const write_request = root.lookupType("prometheus.WriteRequest")

    const writeRequest = write_request.create({
      timeseries: [
        {
          labels: [
            { name: '__name__', value: 'my_app_requests_total' },
            { name: 'instance', value: 'test' },
          ],
          samples: [
            {
              value: 42,
              timestamp: Date.now(),
            },
          ],
        },
      ],
    })

    // chatgpt example : https://chatgpt.com/share/678a6cd1-1074-800e-9cc8-bd1481713dac
    // 2) Encoder en binaire (Protobuf)
    const messageBuffer = write_request.encode(writeRequest).finish()

    // 3) Compresser avec Snappy
    const compressed = await snappy.compress(messageBuffer)

    try {
      // Push data to Mimir
      const mimirUrl = 'https://mimir.france-nuage.fr/api/v1/push' // Replace with your Mimir URL
      const mimirResponse = await axios.post(mimirUrl, compressed, {
        headers: {
          'CF-Access-Client-Id': Env.get('CLOUDFLARE_CLIENT_ID'),
          'CF-Access-Client-Secret': Env.get('CLOUDFLARE_CLIENT_SECRET'),
          'Content-Type': 'application/x-protobuf',
          'Content-Encoding': 'snappy',
          'X-Prometheus-Remote-Write-Version': '0.1.0',
        },
      }.catch()

      console.log('Metrics pushed successfully:', mimirResponse)

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
  }
}
