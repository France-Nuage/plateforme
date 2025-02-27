import vine from '@vinejs/vine'
import { Status } from '#models/infrastructure/instance'

export const createInstanceValidator = vine.compile(
  vine.object({
    zoneId: vine.string().trim().minLength(6),
    name: vine.string().trim().minLength(6),
  })
)

export const queryInstancesValidator = vine.compile(
  vine.object({
    includes: vine.array(vine.string()).optional(),
    page: vine.number().min(1).optional(),
    perPage: vine.number().min(1).max(100).optional(),
  })
)

export const updateInstanceValidator = vine.compile(
  vine.object({
    status: vine.enum(Status).optional(),
  })
)

export const getInstanceCurrentPriceValidator = vine.compile(
  vine.object({
    zoneId: vine.string().uuid(),
    cpu: vine.number(),
    ram: vine.number(),
  })
)
