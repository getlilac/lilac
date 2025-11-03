-- Revert gpus array back to single gpu column
ALTER TABLE cluster_nodes
    RENAME COLUMN gpus TO gpu;

-- Change back to nullable single gpu_configuration
ALTER TABLE cluster_nodes
    ALTER COLUMN gpu DROP NOT NULL,
    ALTER COLUMN gpu DROP DEFAULT,
    ALTER COLUMN gpu TYPE gpu_configuration USING CASE
        WHEN array_length(gpu, 1) > 0 THEN gpu[1]
        ELSE NULL
    END;
