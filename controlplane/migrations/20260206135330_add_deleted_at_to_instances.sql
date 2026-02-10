-- Add soft delete support to instances table
ALTER TABLE instances ADD COLUMN deleted_at timestamptz;
