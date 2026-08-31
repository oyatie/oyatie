use super::*;

impl CloudDcopsCatalog {
    pub fn open_work_order(
        &mut self,
        input: WorkOrderCreate,
    ) -> Result<WorkOrder, CloudDcopsError> {
        let work_order = WorkOrder::new(input)?;
        self.require_active_site(&work_order.site_id.value)?;
        if let Some(equipment_id) = work_order.equipment_id.value.as_ref() {
            let equipment = self
                .equipment
                .get(equipment_id)
                .ok_or(CloudDcopsError::UnknownEquipment)?;
            validate_same_site(&work_order.site_id.value, &equipment.site_id.value)?;
        }
        if self.work_orders.contains_key(&work_order.id.value) {
            return Err(CloudDcopsError::DuplicateWorkOrder);
        }
        self.work_orders
            .insert(work_order.id.value.clone(), work_order.clone());
        Ok(work_order)
    }

    pub fn assign_work_order(
        &mut self,
        work_order_id: &WorkOrderId,
        assigned_to: String,
        updated_at_epoch_seconds: u64,
    ) -> Result<WorkOrder, CloudDcopsError> {
        let current = self
            .work_orders
            .get(work_order_id)
            .ok_or(CloudDcopsError::UnknownWorkOrder)?;
        let next = current.assign(assigned_to, updated_at_epoch_seconds)?;
        self.work_orders.insert(work_order_id.clone(), next.clone());
        Ok(next)
    }

    pub fn start_work_order(
        &mut self,
        work_order_id: &WorkOrderId,
        updated_at_epoch_seconds: u64,
    ) -> Result<WorkOrder, CloudDcopsError> {
        let current = self
            .work_orders
            .get(work_order_id)
            .ok_or(CloudDcopsError::UnknownWorkOrder)?;
        let next = current.start(updated_at_epoch_seconds)?;
        self.work_orders.insert(work_order_id.clone(), next.clone());
        Ok(next)
    }

    pub fn complete_work_order(
        &mut self,
        work_order_id: &WorkOrderId,
        resolution: WorkOrderResolution,
    ) -> Result<WorkOrder, CloudDcopsError> {
        let current = self
            .work_orders
            .get(work_order_id)
            .ok_or(CloudDcopsError::UnknownWorkOrder)?;
        let next = current.complete(resolution)?;
        self.work_orders.insert(work_order_id.clone(), next.clone());
        Ok(next)
    }
}
