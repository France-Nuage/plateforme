import vine from '@vinejs/vine'

export const createAccountValidator = vine.compile(
  vine.object({
    name: vine.string().trim().nullable(),
    currency: vine.string().trim().nullable(),
    organization__id: vine.string().trim().nullable(),
  })
)

export const updateAccountValidator = vine.compile(
  vine.object({
    name: vine.string().trim().minLength(6),
    open: vine.boolean(),
  })
)
