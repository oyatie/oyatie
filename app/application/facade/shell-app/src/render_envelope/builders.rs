use super::types::{
    ApprovalItem, CommunityItem, IntelligenceSuggestion, MessageItem, MetricCard, ModuleCard,
    OntologyFact, ProductActivityStep, ScheduleItem, WorkItem, WorkflowEdge, WorkflowNode,
    WorkflowPreview,
};

pub(super) fn workflow(name: &str, goal: &str, nodes: Vec<WorkflowNode>) -> WorkflowPreview {
    let edges = nodes
        .windows(2)
        .map(|pair| WorkflowEdge {
            from: pair[0].id.clone(),
            to: pair[1].id.clone(),
            label: s("then"),
        })
        .collect();

    WorkflowPreview {
        name: s(name),
        goal: s(goal),
        nodes,
        edges,
    }
}

pub(super) fn activity_step(
    route_key: &str,
    label: &str,
    surface: &str,
    detail: &str,
    target: &str,
    state: &str,
) -> ProductActivityStep {
    ProductActivityStep {
        route_key: s(route_key),
        label: s(label),
        surface: s(surface),
        detail: s(detail),
        target: s(target),
        state: s(state),
    }
}

pub(super) fn metric(label: &str, value: &str, detail: &str) -> MetricCard {
    MetricCard {
        label: s(label),
        value: s(value),
        detail: s(detail),
    }
}

pub(super) fn module(name: &str, group: &str, description: &str, action_label: &str) -> ModuleCard {
    ModuleCard {
        name: s(name),
        group: s(group),
        description: s(description),
        action_label: s(action_label),
    }
}

pub(super) fn work(title: &str, detail: &str, priority: &str) -> WorkItem {
    WorkItem {
        title: s(title),
        detail: s(detail),
        priority: s(priority),
    }
}

pub(super) fn schedule(time: &str, title: &str, detail: &str) -> ScheduleItem {
    ScheduleItem {
        time: s(time),
        title: s(title),
        detail: s(detail),
    }
}

pub(super) fn message(from: &str, channel: &str, preview: &str) -> MessageItem {
    MessageItem {
        from: s(from),
        channel: s(channel),
        preview: s(preview),
    }
}

pub(super) fn community(space: &str, topic: &str, activity: &str) -> CommunityItem {
    CommunityItem {
        space: s(space),
        topic: s(topic),
        activity: s(activity),
    }
}

pub(super) fn approval(title: &str, requester: &str, risk_note: &str) -> ApprovalItem {
    ApprovalItem {
        title: s(title),
        requester: s(requester),
        risk_note: s(risk_note),
    }
}

pub(super) fn node(
    id: &str,
    label: &str,
    kind: &str,
    x: i32,
    y: i32,
    explanation: &str,
) -> WorkflowNode {
    WorkflowNode {
        id: s(id),
        label: s(label),
        kind: s(kind),
        x,
        y,
        explanation: s(explanation),
    }
}

pub(super) fn fact(entity: &str, relation: &str, access_reason: &str) -> OntologyFact {
    OntologyFact {
        entity: s(entity),
        relation: s(relation),
        access_reason: s(access_reason),
    }
}

pub(super) fn suggestion(title: &str, body: &str, guardrail: &str) -> IntelligenceSuggestion {
    IntelligenceSuggestion {
        title: s(title),
        body: s(body),
        guardrail: s(guardrail),
    }
}

pub(super) fn s(value: &str) -> String {
    value.to_string()
}
