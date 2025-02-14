import axios from 'axios'
import config from '@adonisjs/core/services/config'

export const proxmoxApi = {
  node: {
    qemu: {
      async create(
        node: { vmid: string; nodeName: string; token: string; url: string },
        options: { name: string; [_: string]: string | number | boolean }
      ) {
        try {
          const response = await axios.post(
            `${node.url}/api2/json/nodes/${node.nodeName}/qemu`,
            {
              ...options,
              vmid: Number.parseInt(node.vmid),
            },
            {
              headers: {
                'Authorization': node.token,
                'CF-Access-Client-Id': config.get('dev.cloudflare.clientId'),
                'CF-Access-Client-Secret': config.get('dev.cloudflare.clientSecret'),
              },
            }
          )
          return response.data.data
        } catch (e) {
          throw new Error(e)
        }
      },
      async get({
        url,
        nodeName,
        token,
        vmid,
      }: {
        url: string
        nodeName: string
        token: string
        vmid: string
      }) {
        try {
          const response = await axios.get(`${url}/api2/json/nodes/${nodeName}/qemu/${vmid}`, {
            headers: {
              'Authorization': token,
              'CF-Access-Client-Id': config.get('dev.cloudflare.clientId'),
              'CF-Access-Client-Secret': config.get('dev.cloudflare.clientSecret'),
            },
          })
          return response.data.data
        } catch (e) {
          throw new Error(e)
        }
      },
      async destroy({
        url,
        nodeName,
        token,
        vmid,
      }: {
        url: string
        nodeName: string
        token: string
        vmid: string
      }) {
        try {
          const response = await axios.delete(`${url}/api2/json/nodes/${nodeName}/qemu/${vmid}`, {
            headers: {
              'Authorization': token,
              'CF-Access-Client-Id': config.get('dev.cloudflare.clientId'),
              'CF-Access-Client-Secret': config.get('dev.cloudflare.clientSecret'),
            },
          })
          return response.data.data
        } catch (e) {
          throw new Error(e)
        }
      },
      status: {
        async current({
          url,
          nodeName,
          token,
          vmid,
        }: {
          url: string
          nodeName: string
          token: string
          vmid: string
        }) {
          try {
            const response = await axios.get(
              `${url}/api2/json/nodes/${nodeName}/qemu/${vmid}/status/current`,
              {
                headers: {
                  'Authorization': token,
                  'CF-Access-Client-Id': config.get('dev.cloudflare.clientId'),
                  'CF-Access-Client-Secret': config.get('dev.cloudflare.clientSecret'),
                },
              }
            )

            return response.data.data
          } catch (e) {
            return new Error(e)
          }
        },
        async change({
          url,
          nodeName,
          token,
          vmid,
          status,
        }: {
          url: string
          nodeName: string
          token: string
          vmid: string
          status: 'start' | 'stop' | 'reset'
        }) {
          try {
            return axios.post(`${url}/api2/json/nodes/${nodeName}/qemu/${vmid}/${status}`, {
              headers: {
                'Authorization': token,
                'CF-Access-Client-Id': config.get('dev.cloudflare.clientId'),
                'CF-Access-Client-Secret': config.get('dev.cloudflare.clientSecret'),
              },
            })
          } catch (e) {
            return new Error(e)
          }
        },
      },
    },
  },
}
