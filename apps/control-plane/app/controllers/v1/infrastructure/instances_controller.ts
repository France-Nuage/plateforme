import type { HttpContext } from '@adonisjs/core/http'
import InstancePolicy from '#policies/infrastructure/instance_policy'
import instance_service from '#services/v1/infrastructure/instance_service'
import {
  createInstanceValidator,
  getInstanceCurrentPriceValidator,
  queryInstancesValidator,
  updateInstanceValidator,
} from '#validators/v1/infrastructure/instance'
import Node from '#models/infrastructure/node'
import Instance, { Status } from '#models/infrastructure/instance'

export default class InstancesController {
  /**
   * Display a list of resource
   */
  async index({ response, request, bouncer }: HttpContext) {
    await bouncer.with(InstancePolicy).authorize('index')
    const validated = await queryInstancesValidator.validate(request.qs())

    return response.ok(await instance_service.list(validated))
  }

  /**
   * Handle form submission for the create action
   */
  async store({ response, request, bouncer }: HttpContext) {
    await bouncer.with(InstancePolicy).authorize('store')
    const payload = await request.validateUsing(createInstanceValidator)
    const instance = await instance_service.create({ ...payload })
    return response.created(instance)
  }

  /**
   * Show individual record
   */
  async show({ response, params, request, bouncer }: HttpContext) {
    await bouncer.with(InstancePolicy).authorize('show')
    return response.ok(await instance_service.get(params.id, request.qs().includes))
  }

  /**
   * Handle form submission for the edit action
   */
  async update({ response, params, request, bouncer }: HttpContext) {
    await bouncer.with(InstancePolicy).authorize('update')
    const instance = await instance_service.get(params.id, request.qs().includes)
    const payload = await request.validateUsing(updateInstanceValidator)
    const updated = await instance_service.update(instance, payload)
    return response.ok(updated)
  }

  /**
   * Delete record
   */
  async destroy({ response, params }: HttpContext) {
    const instance = await Instance.findOrFail(params.node_id)
    const node = await Node.findOrFail(instance.nodeId)
    node.load('cluster')

    await node.api().instance(instance.pveVmId).delete()

    instance.status = Status.Deleting
    await instance.save()

    return response.ok(instance)
  }

  /**
   * Start instance
   */
  async start({ response, params }: HttpContext) {
    const instance = await Instance.findOrFail(params.node_id)
    const node = await Node.findOrFail(instance.nodeId)
    node.load('cluster')

    await node.api().instance(instance.pveVmId).start()

    instance.status = Status.Staging
    await instance.save()

    return response.ok(instance)
  }

  /**
   * Stop instance
   */
  async stop({ response, params }: HttpContext) {
    const instance = await Instance.findOrFail(params.node_id)
    const node = await Node.findOrFail(instance.nodeId)
    node.load('cluster')

    await node.api().instance(instance.pveVmId).stop()

    instance.status = Status.Stopping
    await instance.save()
    return response.ok(instance)
  }

  async getPrice({ response, request }: HttpContext) {
    const payload = await request.validateUsing(getInstanceCurrentPriceValidator)

    return response.ok(await instance_service.getCurrentPrice({ ...payload }))
  }
}
