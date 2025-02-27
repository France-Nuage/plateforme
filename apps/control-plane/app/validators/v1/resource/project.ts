import vine from '@vinejs/vine'

export const createProjectValidator = vine.compile(
  vine.object({
    name: vine.string().trim().minLength(6),
    folderId: vine.string().trim().minLength(6),
  })
)

export const updateProjectValidator = vine.compile(
  vine.object({
    name: vine.string().trim().minLength(6),
  })
)
