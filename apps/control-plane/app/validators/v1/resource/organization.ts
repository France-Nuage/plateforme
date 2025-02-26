import vine from '@vinejs/vine'

export const createOrganizationValidator = vine.compile(
  vine.object({
    name: vine.string().trim().minLength(6),
  })
)

export const updateOrganizationValidator = vine.compile(
  vine.object({
    name: vine.string().trim().minLength(6),
  })
)
