import vine from '@vinejs/vine'

export const createFolderValidator = vine.compile(
  vine.object({
    name: vine.string().trim().minLength(6),
    organizationId: vine.string().trim(),
  })
)

export const updateFolderValidator = vine.compile(
  vine.object({
    name: vine.string().trim().minLength(6),
    organizationId: vine.string().trim().optional(),
  })
)
