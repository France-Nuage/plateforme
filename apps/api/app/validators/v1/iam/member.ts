import vine from '@vinejs/vine'

export const createMemberValidator = vine.compile(
  vine.object({
    email: vine.string().email(),
    roles: vine.array(vine.string()),
  })
)
