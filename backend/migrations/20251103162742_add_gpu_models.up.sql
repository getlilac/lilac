-- Add missing GPU models to the gpu_model enum
ALTER TYPE gpu_model ADD VALUE IF NOT EXISTS 'A30';
ALTER TYPE gpu_model ADD VALUE IF NOT EXISTS 'L40';
