import vine from '@vinejs/vine'

export const MetricsValidator = vine.compile(
  vine.object({
    ip_address: vine
      .string()
      .trim()
      .regex(
        new RegExp(
          /**
           * IPv4 Regex Explanation:
           * - Matches 4 groups of numbers (0-255) separated by dots.
           * - Prevents invalid values like 999.999.999.999.
           */
          /^(25[0-5]|2[0-4][0-9]|1?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|1?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|1?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|1?[0-9][0-9]?)$/
        )
      ),

    hostname: vine.string().trim().maxLength(255),

    total_memory: vine.number().positive(),

    cpu_count: vine.number().positive().min(1).max(128), // ✅ Fixed range issue

    disk_space: vine.number().positive(),

    os: vine.string().trim().maxLength(100),

    os_version: vine.string().trim().maxLength(50),

    installed_packages: vine.array(vine.string().trim().maxLength(255)),
  })
)
