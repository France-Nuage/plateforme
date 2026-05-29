CREATE VIEW managed.service_instance_view AS
SELECT si.id,
       si.service_id,
       si.version_id,
       si.project_id,
       si.organization_id,
       si.namespace,
       si.release_name,
       si.user_values,
       abs.name AS status,
       si.created_at
FROM managed.service_instance si
JOIN lib_fsm.state_machine sm ON sm.state_machine__id = si.status
JOIN lib_fsm.abstract_state abs ON abs.abstract_state__id = sm.abstract_state__id;
