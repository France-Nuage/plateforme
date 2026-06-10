-- atlas:nolint
-- Links each managed service instance to the Kubernetes cluster hosting it.
--
-- The cluster is resolved at instance creation by matching the labels required
-- by the service deploy_target against the labels carried by healthy clusters
-- (see 20260610120000_create_kubernetes_cluster_labels.sql). Storing the id on
-- the instance keeps every later operation (upgrade, delete) on the cluster
-- the release actually lives on, even if the cluster labels change afterwards.
--
-- ON DELETE RESTRICT: a cluster cannot be removed while instances still run on
-- it. The application layer (KubernetesClusters::delete_cluster) returns a
-- typed ClusterHasInstances error before reaching this constraint; the
-- RESTRICT rule is the database-level backstop.

ALTER TABLE managed.service_instance
    ADD COLUMN cluster_id UUID NOT NULL;

ALTER TABLE managed.service_instance
    ADD CONSTRAINT service_instance_cluster_id_fkey
        FOREIGN KEY (cluster_id)
        REFERENCES kubernetes.cluster (id)
        ON UPDATE NO ACTION
        ON DELETE RESTRICT;

-- Speeds up the hosted-instances lookup performed when deleting a cluster.
CREATE INDEX idx_managed_service_instance_cluster
    ON managed.service_instance (cluster_id);
