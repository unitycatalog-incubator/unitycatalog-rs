-- no-transaction
-- Add the `tagged` / `tagged_by` association labels for entity tag assignments.
--
-- `ALTER TYPE ... ADD VALUE` cannot run inside a transaction block, hence the
-- `-- no-transaction` directive above (honored by sqlx's migrator).
--
-- A tag assignment is modeled as an association edge from the tagged entity to its
-- TagPolicy (`tagged`), with the inverse (`tagged_by`) recorded automatically. The
-- tag value is carried in the association's `properties` JSON. Values are snake_case
-- to match the `AssociationLabel` enum's serialization.
ALTER TYPE association_label ADD VALUE IF NOT EXISTS 'tagged';
ALTER TYPE association_label ADD VALUE IF NOT EXISTS 'tagged_by';
