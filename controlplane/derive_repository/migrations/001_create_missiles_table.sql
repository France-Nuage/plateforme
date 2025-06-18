CREATE TABLE missiles(
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  damage INTEGER NOT NULL
);
