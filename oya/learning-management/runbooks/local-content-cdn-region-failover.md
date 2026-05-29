# Learning Management Runbook: Content Cdn Region Failover

Service: learning-management  
Surface: local operational primitive suite  
Primary SLO: course-progress-freshness  
Domain focus: course, cohort, assessment, content

## Trigger
- Alert `learning-management-course-progress-freshness` burns above the 2x multi-window threshold.
- Operator report names LearningCohort state drift for tenant-scoped course delivery, cohort enrollment, assessment submission, certificate issuance, and learning content operations.
- Audit chain shows denied or missing event class for `course.published`.

## Confirm
1. Query `sum(rate(oya_learning_management_course_progress_freshness_total[5m])) by (tenant_id, cell_tier)` and identify the affected tenant and cell.
2. Compare `good_total` against `total` for the same metric and confirm the burn is not a dashboard-only gap.
3. Inspect the latest policy decision for action `course.publish` and data class `course_content`.
4. Verify the latest domain event on `learning-management.local-ops.v1` carries `audit_event_id` and tenant scope.

## Mitigate
1. Freeze new high-volume writes for the affected tenant using the local Cedar action `cohort.enroll` when burn exceeds 4x.
2. Shift traffic away from the unhealthy cell for the `content-api` workload.
3. Replay only idempotent events with matching `tenant_id`, `resource_id`, and `audit_event_id`.
4. Re-run the policy check endpoint before reopening operator writes.

## Recover
- Restore normal admission when course-progress-freshness is below 1x burn for two consecutive 30 minute windows.
- Backfill missing audit evidence before resolving the incident.
- Attach dashboard snapshot `learning-management-local-course-progress-freshness` and policy file name to the ticket.

## Escalate
Escalate to the service owner when regulated data class `course_content` is affected for more than 15 minutes or when breakglass was used.
