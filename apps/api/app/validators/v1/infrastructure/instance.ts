import vine from '@vinejs/vine'

export const createInstanceValidator = vine.compile(
  vine.object({
    zoneId: vine.string().trim().minLength(6),
    name: vine.string().trim().minLength(6),
  })
)

export const updateInstanceValidator = vine.compile(
  vine.object({
    // name: vine.string().trim().minLength(6),
  })
)

export const getInstanceCurrentPriceValidator = vine.compile(
  vine.object({
    zoneId: vine.string().uuid(),
    cpu: vine.number(),
    ram: vine.number(),
  })
)
