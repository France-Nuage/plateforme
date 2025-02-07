import axios from 'axios'
import env from '#start/env'

export const proxmoxApi = {
  node: {
    qemu: {
      async create(
        config: { vmid: string; nodeName: string; token: string; url: string },
        options: { name: string; [_: string]: string | number | boolean }
      ) {
        try {
          const response = await axios.post(
            `${config.url}/api2/json/nodes/${config.nodeName}/qemu`,
            {
              ...options,
              vmid: Number.parseInt(config.vmid),
            },
            {
              headers: {
                Authorization: config.token,
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
              Authorization: token,
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
              Authorization: token,
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
                  'CF-Access-Client-Id': env.get('CLOUDFLARE_ACCESS_CLIENT_ID'),
                  'CF-Access-Client-Secret': env.get('CLOUDFLARE_ACCESS_CLIENT_SECRET'),
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
                Authorization: token,
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
