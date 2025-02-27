import vine from '@vinejs/vine'

export const createUserValidator = vine.compile(
  vine.object({
    email: vine
      .string()
      .email()
      .unique(async (db, value) => {
        const user = await db.from('member.users').where('email', value).first()
        return !user
      }),
    lastname: vine.string().trim().nullable(),
    firstname: vine.string().trim().nullable(),
    password: vine.string().minLength(8).maxLength(512),
  })
)

export const updateUserValidator = vine.compile(
  vine.object({
    lastname: vine.string().trim().minLength(6),
    firstname: vine.string().trim().minLength(6),
  })
)

/**
 * Validates the post's update action
 */
export const loginUserValidator = vine.compile(
  vine.object({
    email: vine.string().email(),
    password: vine.string(),
  })
)

export const resetPasswordRequestValidator = vine.compile(
  vine.object({
    email: vine.string().email(),
  })
)

export const resetPasswordValidator = vine.compile(
  vine.object({
    token: vine.string(),
    password: vine.string().minLength(8).maxLength(512),
  })
)
