-- Enhance "invitations" table for full lifecycle management
-- Adds email-based invitations, token generation, expiration, and role assignment

-- Email for invitation by email (user may not exist yet)
ALTER TABLE "public"."invitations" ADD COLUMN IF NOT EXISTS "email" text;

-- Role ID to assign when invitation is accepted
ALTER TABLE "public"."invitations" ADD COLUMN IF NOT EXISTS "role_id" uuid;

-- Token for secure invitation URLs
ALTER TABLE "public"."invitations" ADD COLUMN IF NOT EXISTS "token" text UNIQUE;

-- Expiration timestamp for the invitation
ALTER TABLE "public"."invitations" ADD COLUMN IF NOT EXISTS "expires_at" timestamptz;

-- Timestamp when the invitation was answered (accepted or declined)
ALTER TABLE "public"."invitations" ADD COLUMN IF NOT EXISTS "answered_at" timestamptz;

-- Make user_id nullable for email-based invitations where user doesn't exist yet
ALTER TABLE "public"."invitations" ALTER COLUMN "user_id" DROP NOT NULL;

-- Index for looking up invitations by token
CREATE INDEX IF NOT EXISTS idx_invitations_token ON "public"."invitations" ("token") WHERE "token" IS NOT NULL;

-- Index for looking up invitations by email
CREATE INDEX IF NOT EXISTS idx_invitations_email ON "public"."invitations" ("email") WHERE "email" IS NOT NULL;

-- Index for cleaning up expired invitations
CREATE INDEX IF NOT EXISTS idx_invitations_expires_at ON "public"."invitations" ("expires_at") WHERE "state" = 'PENDING';
