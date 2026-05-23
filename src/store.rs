use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use reqwest::Client;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::*;

pub struct A2aStore {
    client: Client,
    agents: Arc<RwLock<HashMap<String, RemoteAgent>>>,
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    push_configs: Arc<RwLock<Vec<PushNotificationConfig>>>,
}

impl A2aStore {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
            agents: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            push_configs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn resolve_card(&self, base_url: &str) -> Result<AgentCard, String> {
        let base = base_url.trim_end_matches('/');
        // Try standard path first, then deprecated path
        for path in ["/.well-known/agent-card.json", "/.well-known/agent.json"] {
            let url = format!("{}{}", base, path);
            match self.client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<AgentCard>().await {
                        Ok(card) => return Ok(card),
                        Err(_) => continue,
                    }
                }
                _ => continue,
            }
        }
        Err(format!("No agent card found at {}", base_url))
    }

    pub async fn register_agent(&self, base_url: String, card: AgentCard) -> RemoteAgent {
        let agent = RemoteAgent {
            agent_id: format!("agent_{}", Uuid::new_v4().simple()),
            name: card.name.clone(),
            base_url,
            status: "connected".into(),
            card: Some(card),
            auth_mode: None,
            last_seen_at: Some(Utc::now()),
            created_at: Utc::now(),
        };
        self.agents.write().await.insert(agent.agent_id.clone(), agent.clone());
        agent
    }

    pub async fn list_agents(&self) -> Vec<RemoteAgent> {
        self.agents.read().await.values().cloned().collect()
    }

    pub async fn send_message(&self, agent_id: &str, text: &str) -> Result<Task, String> {
        let agents = self.agents.read().await;
        let agent = agents.get(agent_id).ok_or_else(|| format!("Agent not found: {}", agent_id))?;
        let card = agent.card.as_ref().ok_or("No agent card")?;

        let message_id = Uuid::new_v4().to_string();
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "message/send",
            "params": {
                "message": {
                    "role": "user",
                    "parts": [{"text": text}],
                    "messageId": message_id,
                }
            },
            "id": message_id,
        });

        let resp = self.client.post(&card.url).json(&request).send().await.map_err(|e| e.to_string())?;
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

        if let Some(error) = body.get("error") {
            return Err(format!("RPC error: {}", error));
        }

        // Parse the A2A response - handle both task and message formats
        let result = &body["result"];
        let task_id = result["id"].as_str()
            .or(result["taskId"].as_str())
            .unwrap_or("unknown")
            .to_string();

        let state = result["status"]["state"].as_str().unwrap_or("completed").to_string();

        // Extract text from artifacts (standard A2A format) or history
        let mut history = Vec::new();
        history.push(Message {
            role: "user".into(),
            parts: vec![Part::Text { text: text.to_string() }],
            message_id: Some(message_id.clone()),
            task_id: Some(task_id.clone()),
        });

        // Try artifacts first (Google ADK / standard A2A format)
        if let Some(artifacts) = result["artifacts"].as_array() {
            for artifact in artifacts {
                if let Some(parts) = artifact["parts"].as_array() {
                    for part in parts {
                        let text_content = part["text"].as_str()
                            .or(part.get("text").and_then(|t| t.as_str()))
                            .unwrap_or("");
                        if !text_content.is_empty() {
                            history.push(Message {
                                role: "agent".into(),
                                parts: vec![Part::Text { text: text_content.to_string() }],
                                message_id: None,
                                task_id: Some(task_id.clone()),
                            });
                        }
                    }
                }
            }
        }

        // Try history (LangGraph / simple A2A format)
        if let Some(hist) = result["history"].as_array() {
            for msg in hist {
                if msg["role"].as_str() == Some("agent") {
                    if let Some(parts) = msg["parts"].as_array() {
                        for part in parts {
                            let text_content = part["text"].as_str().unwrap_or("");
                            if !text_content.is_empty() && !history.iter().any(|h| h.role == "agent") {
                                history.push(Message {
                                    role: "agent".into(),
                                    parts: vec![Part::Text { text: text_content.to_string() }],
                                    message_id: msg["messageId"].as_str().map(String::from),
                                    task_id: Some(task_id.clone()),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Try status.message (input-required format)
        if let Some(status_msg) = result["status"]["message"].as_object() {
            if let Some(parts) = status_msg.get("parts").and_then(|p| p.as_array()) {
                for part in parts {
                    let text_content = part["text"].as_str().unwrap_or("");
                    if !text_content.is_empty() && !history.iter().any(|h| h.role == "agent") {
                        history.push(Message {
                            role: "agent".into(),
                            parts: vec![Part::Text { text: text_content.to_string() }],
                            message_id: None,
                            task_id: Some(task_id.clone()),
                        });
                    }
                }
            }
        }

        let task = Task {
            id: task_id,
            context_id: result["contextId"].as_str().map(String::from),
            status: TaskStatus { state: match state.as_str() {
                "submitted" => TaskState::Submitted,
                "working" => TaskState::Working,
                "input_required" | "input-required" => TaskState::InputRequired,
                "failed" => TaskState::Failed,
                "canceled" => TaskState::Canceled,
                _ => TaskState::Completed,
            }, message: None },
            history: Some(history),
        };
        self.tasks.write().await.insert(task.id.clone(), task.clone());
        Ok(task)
    }

    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        self.tasks.read().await.get(task_id).cloned()
    }

    pub async fn list_tasks(&self) -> Vec<Task> {
        self.tasks.read().await.values().cloned().collect()
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus { state: TaskState::Canceled, message: Some("Canceled by operator".into()) };
            Ok(())
        } else {
            Err(format!("Task not found: {}", task_id))
        }
    }

    pub async fn add_push_config(&self, agent_id: String, webhook_url: String, events: Vec<String>) -> PushNotificationConfig {
        let config = PushNotificationConfig {
            config_id: format!("push_{}", Uuid::new_v4().simple()),
            agent_id, webhook_url, events, active: true, created_at: Utc::now(),
        };
        self.push_configs.write().await.push(config.clone());
        config
    }

    pub async fn list_push_configs(&self) -> Vec<PushNotificationConfig> {
        self.push_configs.read().await.clone()
    }
}
