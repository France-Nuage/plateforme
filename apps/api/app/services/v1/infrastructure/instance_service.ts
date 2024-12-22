import axios from 'axios'
import Zone from '#models/infrastructure/zone'
import RequestQueryBuilder from '../../../utils/RequestQueryBuilder.js'
import Instance from '#models/infrastructure/instance'
import Folder from "#models/resource/folder";

const getNextVMID = async (url: string, token: string) => {
  try {
    const response = await axios.get(`${url}/api2/json/cluster/nextid`, {
      headers: {
        Authorization: token,
      },
    })
    return response.data.data
  } catch (e) {
    throw new Error('Could not get next VMID')
    // throw new Error(e)
  }
}

const createVM = async (
  config: { vmid: string; nodeName: string; token: string; url: string },
  options: { name: string; [_: string]: string | number | boolean }
) => {
  try {
    const response = await axios.post(
      `${config.url}/api2/json/nodes/${config.nodeName}/qemu`,
      {
        ...options,
        vmid: parseInt(config.vmid),
      },
      {
        headers: {
          Authorization: config.token,
        },
      }
    )
    return response.data.data
  } catch (e) {
    console.log(
      `${config.url}/api2/json/nodes/${config.nodeName}/qemu`,
      {
        ...options,
        vmid: parseInt(config.vmid),
      },
      {
        headers: {
          Authorization: config.token,
        },
      }
    )
    throw new Error(e)
  }
}

export default {
  list: async function (includes: Array<string>) {
    return new RequestQueryBuilder(Instance.query())
      .withIncludes(includes)
      .withPagination(1, 10)
      .apply()
  },
  get: async function (id: string, includes: Array<string>) {
    return new RequestQueryBuilder(Instance.query())
      .withIncludes(includes)
      .applyWhere([['id', '=', id]])
      .firstOrFail()
  },
  create: async function ({ zoneId, name }: { zoneId: string; name: string }) {
    const zone = await new RequestQueryBuilder(Zone.query())
      .withIncludes(['clusters.nodes'])
      .applyWhere([['id', zoneId]])
      .firstOrFail()

    if (!zone.clusters[0]?.nodes[0]) {
      return new Error('No nodes available in the zone')
    }

    const node = zone.clusters[0].nodes[0]

    const vmid = await getNextVMID(node.url, node.token)
    const vm = await createVM(
      {
        vmid,
        nodeName: node.name,
        token: node.token,
        url: node.url,
      },
      {
        name,
      }
    )
    return await Instance.create({
      name: name,
      pveVmId: vmid,
      nodeId: node.id,
    })
  },
}
