#[derive(Clone, Debug, Eq, PartialEq)]
struct DependencyClosureIndexV1 {
    component_by_node: Box<[u32]>,
    dependent_offsets: Box<[usize]>,
    dependent_components: Box<[u32]>,
    topological_components: Box<[u32]>,
}

impl DependencyClosureIndexV1 {
    fn component_count(&self) -> usize {
        self.topological_components.len()
    }

    fn component_for_node(&self, node_index: usize) -> usize {
        self.component_by_node[node_index] as usize
    }

    fn dependent_components(&self, component: usize) -> &[u32] {
        &self.dependent_components
            [self.dependent_offsets[component]..self.dependent_offsets[component + 1]]
    }

    fn topological_components(&self) -> &[u32] {
        &self.topological_components
    }
}

#[derive(Clone, Copy)]
struct DependencyDfsFrameV1 {
    node: usize,
    next_edge: usize,
    edge_end: usize,
}

struct DependencyStrongComponentStateV1 {
    discovery: Vec<usize>,
    lowlink: Vec<usize>,
    active: Vec<bool>,
    component_by_node: Vec<u32>,
    active_nodes: Vec<usize>,
    frames: Vec<DependencyDfsFrameV1>,
    next_discovery: usize,
    component_count: usize,
}

impl DependencyStrongComponentStateV1 {
    fn try_new(node_count: usize) -> Result<Self, LifecycleFailureV1> {
        Ok(Self {
            discovery: lifecycle_try_filled_vec(node_count, usize::MAX)?,
            lowlink: lifecycle_try_filled_vec(node_count, usize::MAX)?,
            active: lifecycle_try_filled_vec(node_count, false)?,
            component_by_node: lifecycle_try_filled_vec(node_count, u32::MAX)?,
            active_nodes: lifecycle_try_vec(node_count)?,
            frames: lifecycle_try_vec(node_count)?,
            next_discovery: 0,
            component_count: 0,
        })
    }

    fn discover(
        &mut self,
        node: usize,
        reverse: &ReverseDependencyIndexV1,
    ) -> Result<(), LifecycleFailureV1> {
        self.discovery[node] = self.next_discovery;
        self.lowlink[node] = self.next_discovery;
        self.next_discovery = self
            .next_discovery
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        self.active[node] = true;
        self.active_nodes.push(node);
        self.frames.push(DependencyDfsFrameV1 {
            node,
            next_edge: reverse.offsets[node],
            edge_end: reverse.offsets[node + 1],
        });
        Ok(())
    }

    fn complete_component(&mut self, root: usize) -> Result<(), LifecycleFailureV1> {
        loop {
            let member = self.active_nodes.pop().ok_or_else(lifecycle_internal)?;
            self.active[member] = false;
            self.component_by_node[member] = lifecycle_u32_index(self.component_count)?;
            if member == root {
                break;
            }
        }
        self.component_count = self
            .component_count
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
        Ok(())
    }
}

fn build_dependency_closure_index<C>(
    node_count: usize,
    reverse: &ReverseDependencyIndexV1,
    control: &mut DependencyGraphConstructionControlV1<C>,
) -> Result<DependencyClosureIndexV1, LifecycleFailureV1>
where
    C: FnMut(DependencyGraphConstructionProgressV1) -> LifecycleControlDecisionV1,
{
    let (component_by_node, component_count) =
        dependency_strong_components(node_count, reverse, control)?;
    let pairs = dependency_component_pairs(&component_by_node, reverse, control)?;
    let (dependent_offsets, dependent_components) =
        dependency_component_adjacency(component_count, &pairs)?;
    let topological_components = dependency_component_topology(
        component_count,
        &pairs,
        &dependent_offsets,
        &dependent_components,
        control,
    )?;
    Ok(DependencyClosureIndexV1 {
        component_by_node: component_by_node.into_boxed_slice(),
        dependent_offsets: dependent_offsets.into_boxed_slice(),
        dependent_components: dependent_components.into_boxed_slice(),
        topological_components: topological_components.into_boxed_slice(),
    })
}

fn dependency_strong_components<C>(
    node_count: usize,
    reverse: &ReverseDependencyIndexV1,
    control: &mut DependencyGraphConstructionControlV1<C>,
) -> Result<(Vec<u32>, usize), LifecycleFailureV1>
where
    C: FnMut(DependencyGraphConstructionProgressV1) -> LifecycleControlDecisionV1,
{
    let mut state = DependencyStrongComponentStateV1::try_new(node_count)?;

    for start in 0..node_count {
        if state.discovery[start] != usize::MAX {
            continue;
        }
        state.discover(start, reverse)?;
        while !state.frames.is_empty() {
            let (node, next_edge, edge_end) = {
                let frame = state.frames.last().ok_or_else(lifecycle_internal)?;
                (frame.node, frame.next_edge, frame.edge_end)
            };
            if next_edge < edge_end {
                state
                    .frames
                    .last_mut()
                    .ok_or_else(lifecycle_internal)?
                    .next_edge += 1;
                let dependent = reverse.dependents[next_edge];
                control.record_work()?;
                if state.discovery[dependent] == usize::MAX {
                    state.discover(dependent, reverse)?;
                } else if state.active[dependent] {
                    state.lowlink[node] = state.lowlink[node].min(state.discovery[dependent]);
                }
                continue;
            }

            state.frames.pop().ok_or_else(lifecycle_internal)?;
            if let Some(parent) = state.frames.last().map(|frame| frame.node) {
                state.lowlink[parent] = state.lowlink[parent].min(state.lowlink[node]);
            }
            if state.lowlink[node] == state.discovery[node] {
                state.complete_component(node)?;
            }
            control.record_work()?;
        }
    }
    if state.active_nodes.is_empty()
        && state
            .component_by_node
            .iter()
            .all(|component| *component != u32::MAX)
    {
        control.checkpoint_and_reset()?;
        Ok((state.component_by_node, state.component_count))
    } else {
        Err(lifecycle_internal())
    }
}

fn dependency_component_pairs<C>(
    component_by_node: &[u32],
    reverse: &ReverseDependencyIndexV1,
    control: &mut DependencyGraphConstructionControlV1<C>,
) -> Result<Vec<(u32, u32)>, LifecycleFailureV1>
where
    C: FnMut(DependencyGraphConstructionProgressV1) -> LifecycleControlDecisionV1,
{
    let mut pairs = lifecycle_try_vec(reverse.dependents.len())?;
    for dependency in 0..component_by_node.len() {
        let source = component_by_node[dependency];
        for position in reverse.offsets[dependency]..reverse.offsets[dependency + 1] {
            let destination = component_by_node[reverse.dependents[position]];
            if source != destination {
                pairs.push((source, destination));
            }
            control.record_work()?;
        }
    }
    control.checkpoint_and_reset()?;
    pairs.sort_unstable();
    pairs.dedup();
    control.checkpoint_and_reset()?;
    Ok(pairs)
}

fn dependency_component_adjacency(
    component_count: usize,
    pairs: &[(u32, u32)],
) -> Result<(Vec<usize>, Vec<u32>), LifecycleFailureV1> {
    let mut counts = lifecycle_try_filled_vec(component_count, 0_usize)?;
    for (source, _) in pairs {
        let source = *source as usize;
        counts[source] = counts[source]
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
    }
    let mut offsets = lifecycle_try_vec(component_count + 1)?;
    offsets.push(0_usize);
    for count in counts {
        let next = offsets
            .last()
            .copied()
            .and_then(|offset| offset.checked_add(count))
            .ok_or_else(lifecycle_bounds)?;
        offsets.push(next);
    }
    let mut dependents = lifecycle_try_vec(pairs.len())?;
    dependents.extend(pairs.iter().map(|(_, destination)| *destination));
    Ok((offsets, dependents))
}

fn dependency_component_topology<C>(
    component_count: usize,
    pairs: &[(u32, u32)],
    dependent_offsets: &[usize],
    dependent_components: &[u32],
    control: &mut DependencyGraphConstructionControlV1<C>,
) -> Result<Vec<u32>, LifecycleFailureV1>
where
    C: FnMut(DependencyGraphConstructionProgressV1) -> LifecycleControlDecisionV1,
{
    let mut indegrees = lifecycle_try_filled_vec(component_count, 0_usize)?;
    for (_, destination) in pairs {
        let destination = *destination as usize;
        indegrees[destination] = indegrees[destination]
            .checked_add(1)
            .ok_or_else(lifecycle_bounds)?;
    }
    let mut ready = std::collections::BinaryHeap::new();
    ready
        .try_reserve(component_count)
        .map_err(|_| lifecycle_bounds())?;
    for (component, indegree) in indegrees.iter().enumerate() {
        if *indegree == 0 {
            ready.push(std::cmp::Reverse(lifecycle_u32_index(component)?));
        }
    }
    let mut ordered = lifecycle_try_vec(component_count)?;
    while let Some(std::cmp::Reverse(component)) = ready.pop() {
        ordered.push(component);
        let component = component as usize;
        for destination in
            &dependent_components[dependent_offsets[component]..dependent_offsets[component + 1]]
        {
            let destination = *destination as usize;
            indegrees[destination] = indegrees[destination]
                .checked_sub(1)
                .ok_or_else(lifecycle_internal)?;
            if indegrees[destination] == 0 {
                ready.push(std::cmp::Reverse(lifecycle_u32_index(destination)?));
            }
            control.record_work()?;
        }
    }
    control.checkpoint_and_reset()?;
    if ordered.len() != component_count {
        return Err(lifecycle_internal());
    }
    Ok(ordered)
}
