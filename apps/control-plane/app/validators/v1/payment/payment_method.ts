import vine from '@vinejs/vine'

export const createPaymentMethodValidator = vine.compile(
  vine.object({
    name: vine.string().trim().minLength(6),
  })
)

export const updatePaymentMethodValidator = vine.compile(
  vine.object({
    name: vine.string().trim().minLength(6),
  })
)
