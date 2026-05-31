-- atlas:nolint
-- Links each project to the Kubernetes cluster hosting its managed services.
--
-- The column is NULLABLE on purpose:
--   1. Projects created before this migration have no cluster assigned.
--   2. The system 'unattributed' project (created by
--      initialize_root_organization / create_organization) is never deployed to
--      and therefore carries no cluster.
--
-- ON DELETE RESTRICT: a cluster cannot be removed while projects still point to
-- it. The application layer (KubernetesClusters::delete_cluster) returns a
-- typed ClusterHasProjects error before reaching this constraint; the RESTRICT
-- rule is the database-level backstop.

ALTER TABLE projects
    ADD COLUMN cluster_id UUID;

ALTER TABLE projects
    ADD CONSTRAINT projects_cluster_id_fkey
        FOREIGN KEY (cluster_id)
        REFERENCES kubernetes.cluster (id)
        ON UPDATE NO ACTION
        ON DELETE RESTRICT;

-- Speeds up the assigned-projects lookup performed when deleting a cluster.
CREATE INDEX idx_projects_cluster_id ON projects (cluster_id);
