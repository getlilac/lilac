-- Change gpu column from single gpu_configuration to array of gpu_configuration
ALTER TABLE cluster_nodes
    ALTER COLUMN gpu TYPE gpu_configuration[] USING CASE
        WHEN gpu IS NULL THEN ARRAY[]::gpu_configuration[]
        ELSE ARRAY[gpu]
    END;

-- Update the column to be NOT NULL with a default empty array
ALTER TABLE cluster_nodes
    ALTER COLUMN gpu SET DEFAULT ARRAY[]::gpu_configuration[],
    ALTER COLUMN gpu SET NOT NULL;

-- Rename the column to gpus to better reflect that it's now an array
ALTER TABLE cluster_nodes
    RENAME COLUMN gpu TO gpus;
