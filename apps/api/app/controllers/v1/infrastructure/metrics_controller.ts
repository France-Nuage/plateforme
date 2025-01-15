import type {HttpContext} from '@adonisjs/core/http'
import axios from 'axios' // Import HTTP client

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
    const formattedMetrics = {
      metrics: [
        {
          labels: {
            __name__: 'system_metrics',
            ip_address: data.ip_address,
            hostname: data.hostname,
            os: data.os,
            os_version: data.os_version,
          },
          samples: [
            { name: 'total_memory', value: data.total_memory },
            { name: 'cpu_count', value: data.cpu_count },
            { name: 'disk_space', value: data.disk_space },
            { name: 'installed_packages', value: JSON.stringify(data.installed_packages) },
          ],
        },
      ],
    }

    try {
      // Push data to Mimir
      const mimirUrl = 'https://mimir-instance/api/v1/push' // Replace with your Mimir URL
      const mimirResponse = await axios.post(mimirUrl, formattedMetrics, {
        headers: {
          'Content-Type': 'application/json',
        },
      })

      console.log('Metrics pushed successfully:', mimirResponse.data)

      // Respond to the external agent
      return response.ok({
        message: 'Metrics received and pushed successfully',
        pushedData: formattedMetrics,
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
