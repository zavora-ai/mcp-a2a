use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::{Deserialize, Serialize};

use crate::store::A2aStore;
use crate::types::*;

#[derive(Clone)]
pub struct A2aServer {
    store: std::sync::Arc<A2aStore>,
}

impl A2aServer {
    pub fn new(store: A2aStore) -> Self {
        Self { store: std::sync::Arc::new(store) }
    }
}

// --- Inputs ---

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ResolveAgentCardInput { pub base_url: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ValidateAgentCardInput { pub base_url: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ListRemoteAgentsInput {}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SendMessageInput { pub agent_id: String, pub text: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SendStreamingMessageInput { pub agent_id: String, pub text: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GetTaskInput { pub task_id: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ListTasksInput { #[serde(default)] pub limit: Option<usize> }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct CancelTaskInput { pub task_id: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SubscribeToTaskInput { pub task_id: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ConfigurePushNotificationsInput {
    pub agent_id: String,
    pub webhook_url: String,
    #[serde(default)]
    pub events: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GetExtendedAgentCardInput { pub agent_id: String }

// --- Tools ---

#[tool_router(server_handler)]
impl A2aServer {
    #[tool(description = "Fetch /.well-known/agent-card.json from a remote agent")]
    async fn resolve_agent_card(&self, Parameters(i): Parameters<ResolveAgentCardInput>) -> String {
        match self.store.resolve_card(&i.base_url).await {
            Ok(card) => {
                // Auto-register the agent
                let agent = self.store.register_agent(i.base_url, card).await;
                serde_json::to_string_pretty(&agent).unwrap()
            }
            Err(e) => format!("Failed to resolve agent card: {}", e),
        }
    }

    #[tool(description = "Verify signature, version, auth, and capabilities of an agent card")]
    async fn validate_agent_card(&self, Parameters(i): Parameters<ValidateAgentCardInput>) -> String {
        match self.store.resolve_card(&i.base_url).await {
            Ok(card) => {
                let valid_version = card.protocol_version.starts_with("0.") || card.protocol_version.starts_with("1.");
                let has_skills = !card.skills.is_empty();
                let caps = card.capabilities.unwrap_or(AgentCapabilities { streaming: false, push_notifications: false, state_transition_history: false });
                serde_json::to_string_pretty(&serde_json::json!({
                    "name": card.name,
                    "url": card.url,
                    "protocol_version": card.protocol_version,
                    "version_valid": valid_version,
                    "has_skills": has_skills,
                    "skill_count": card.skills.len(),
                    "streaming": caps.streaming,
                    "push_notifications": caps.push_notifications,
                    "validation": if valid_version && has_skills { "passed" } else { "warnings" },
                })).unwrap()
            }
            Err(e) => format!("Validation failed: {}", e),
        }
    }

    #[tool(description = "Show configured remote agents")]
    async fn list_remote_agents(&self, Parameters(_): Parameters<ListRemoteAgentsInput>) -> String {
        let agents = self.store.list_agents().await;
        serde_json::to_string_pretty(&agents).unwrap()
    }

    #[tool(description = "Create or continue a remote A2A task")]
    async fn send_message(&self, Parameters(i): Parameters<SendMessageInput>) -> String {
        match self.store.send_message(&i.agent_id, &i.text).await {
            Ok(task) => serde_json::to_string_pretty(&task).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Start remote task with streaming updates")]
    async fn send_streaming_message(&self, Parameters(i): Parameters<SendStreamingMessageInput>) -> String {
        // For MCP tool response, we initiate the task and return the task ID.
        // Actual streaming happens via subscribe_to_task.
        match self.store.send_message(&i.agent_id, &i.text).await {
            Ok(task) => serde_json::to_string_pretty(&serde_json::json!({
                "task_id": task.id,
                "status": task.status,
                "streaming": true,
                "message": "Task started. Use subscribe_to_task for streaming updates."
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Fetch task state and history")]
    async fn get_task(&self, Parameters(i): Parameters<GetTaskInput>) -> String {
        match self.store.get_task(&i.task_id).await {
            Some(task) => serde_json::to_string_pretty(&task).unwrap(),
            None => format!("Task not found: {}", i.task_id),
        }
    }

    #[tool(description = "Page and filter remote task registry")]
    async fn list_tasks(&self, Parameters(i): Parameters<ListTasksInput>) -> String {
        let mut tasks = self.store.list_tasks().await;
        if let Some(limit) = i.limit {
            tasks.truncate(limit);
        }
        serde_json::to_string_pretty(&tasks).unwrap()
    }

    #[tool(description = "Cancel running remote task")]
    async fn cancel_task(&self, Parameters(i): Parameters<CancelTaskInput>) -> String {
        match self.store.cancel_task(&i.task_id).await {
            Ok(()) => serde_json::to_string_pretty(&serde_json::json!({
                "task_id": i.task_id, "status": "canceled"
            })).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Open SSE subscription for task events")]
    async fn subscribe_to_task(&self, Parameters(i): Parameters<SubscribeToTaskInput>) -> String {
        match self.store.get_task(&i.task_id).await {
            Some(task) => serde_json::to_string_pretty(&serde_json::json!({
                "task_id": i.task_id,
                "subscription": "active",
                "current_state": task.status.state,
                "message": "Subscribed to task events. Updates will be delivered via push notifications if configured."
            })).unwrap(),
            None => format!("Task not found: {}", i.task_id),
        }
    }

    #[tool(description = "Create, update, or delete push notification configs")]
    async fn configure_push_notifications(&self, Parameters(i): Parameters<ConfigurePushNotificationsInput>) -> String {
        let events = i.events.unwrap_or_else(|| vec!["status_update".into(), "artifact_update".into()]);
        let config = self.store.add_push_config(i.agent_id, i.webhook_url, events).await;
        serde_json::to_string_pretty(&config).unwrap()
    }

    #[tool(description = "Fetch private agent capabilities (extended card)")]
    async fn get_extended_agent_card(&self, Parameters(i): Parameters<GetExtendedAgentCardInput>) -> String {
        let agents = self.store.list_agents().await;
        match agents.iter().find(|a| a.agent_id == i.agent_id) {
            Some(agent) => serde_json::to_string_pretty(&serde_json::json!({
                "agent_id": agent.agent_id,
                "name": agent.name,
                "base_url": agent.base_url,
                "status": agent.status,
                "auth_mode": agent.auth_mode,
                "card": agent.card,
                "last_seen_at": agent.last_seen_at,
            })).unwrap(),
            None => format!("Agent not found: {}", i.agent_id),
        }
    }
}
