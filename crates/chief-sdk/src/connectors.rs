//! Kernel connector traits and in-memory stub implementations.

use crate::error::Result;
use async_trait::async_trait;
use serde_json::json;

/// Narrow facade for network operations enforced by the capability broker.
#[async_trait]
pub trait NetworkConnector: Send + Sync {
    async fn http_get(&self, url: &str) -> Result<serde_json::Value>;
    async fn http_post(&self, url: &str, body: serde_json::Value) -> Result<serde_json::Value>;
}

/// Narrow facade for Memory Graph operations.
#[async_trait]
pub trait MemoryConnector: Send + Sync {
    async fn put_node(&self, node: serde_json::Value) -> Result<String>;
    async fn get_node(&self, id: &str) -> Result<Option<serde_json::Value>>;
    async fn query(&self, filter: serde_json::Value) -> Result<Vec<serde_json::Value>>;
}

/// Narrow facade for LLM operations.
#[async_trait]
pub trait InferenceConnector: Send + Sync {
    async fn generate(&self, prompt: &str, tier: &str) -> Result<String>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

/// Narrow facade for event bus operations.
#[async_trait]
pub trait EventBusConnector: Send + Sync {
    async fn emit(&self, topic: &str, payload: serde_json::Value) -> Result<()>;
    async fn subscribe(&self, topic: &str) -> Result<()>;
}

/// In-memory stub for testing.
pub struct InMemoryConnector {
    pub memory: std::sync::Mutex<std::collections::HashMap<String, serde_json::Value>>,
    pub events: std::sync::Mutex<Vec<(String, serde_json::Value)>>,
    pub calls: std::sync::Mutex<Vec<String>>,
}

impl InMemoryConnector {
    pub fn new() -> Self {
        Self {
            memory: std::sync::Mutex::new(std::collections::HashMap::new()),
            events: std::sync::Mutex::new(Vec::new()),
            calls: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemoryConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkConnector for InMemoryConnector {
    async fn http_get(&self, url: &str) -> Result<serde_json::Value> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("http_get: {}", url));
        Ok(json!({"status": 200, "body": "mock"}))
    }

    async fn http_post(&self, url: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("http_post: {} with {:?}", url, body));
        Ok(json!({"status": 200}))
    }
}

#[async_trait]
impl MemoryConnector for InMemoryConnector {
    async fn put_node(&self, node: serde_json::Value) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        self.calls.lock().unwrap().push(format!("put_node: {}", id));
        self.memory.lock().unwrap().insert(id.clone(), node);
        Ok(id)
    }

    async fn get_node(&self, id: &str) -> Result<Option<serde_json::Value>> {
        self.calls.lock().unwrap().push(format!("get_node: {}", id));
        Ok(self.memory.lock().unwrap().get(id).cloned())
    }

    async fn query(&self, _filter: serde_json::Value) -> Result<Vec<serde_json::Value>> {
        let mem = self.memory.lock().unwrap();
        Ok(mem.values().cloned().collect())
    }
}

#[async_trait]
impl InferenceConnector for InMemoryConnector {
    async fn generate(&self, prompt: &str, _tier: &str) -> Result<String> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("generate: {}", prompt));
        Ok("Mock inference response".to_string())
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.calls.lock().unwrap().push(format!("embed: {}", text));
        Ok(vec![0.0; 384])
    }
}

#[async_trait]
impl EventBusConnector for InMemoryConnector {
    async fn emit(&self, topic: &str, payload: serde_json::Value) -> Result<()> {
        self.calls.lock().unwrap().push(format!("emit: {}", topic));
        self.events
            .lock()
            .unwrap()
            .push((topic.to_string(), payload));
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("subscribe: {}", topic));
        Ok(())
    }
}

/// Null connector that records all calls but produces no side effects.
pub struct NullConnector {
    pub calls: std::sync::Mutex<Vec<String>>,
}

impl NullConnector {
    pub fn new() -> Self {
        Self {
            calls: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for NullConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkConnector for NullConnector {
    async fn http_get(&self, url: &str) -> Result<serde_json::Value> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("http_get: {}", url));
        Ok(json!({}))
    }

    async fn http_post(&self, url: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("http_post: {} with {:?}", url, body));
        Ok(json!({}))
    }
}

#[async_trait]
impl MemoryConnector for NullConnector {
    async fn put_node(&self, _node: serde_json::Value) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        self.calls.lock().unwrap().push(format!("put_node: {}", id));
        Ok(id)
    }

    async fn get_node(&self, id: &str) -> Result<Option<serde_json::Value>> {
        self.calls.lock().unwrap().push(format!("get_node: {}", id));
        Ok(None)
    }

    async fn query(&self, _filter: serde_json::Value) -> Result<Vec<serde_json::Value>> {
        self.calls.lock().unwrap().push("query".to_string());
        Ok(Vec::new())
    }
}

#[async_trait]
impl InferenceConnector for NullConnector {
    async fn generate(&self, prompt: &str, _tier: &str) -> Result<String> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("generate: {}", prompt));
        Ok(String::new())
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.calls.lock().unwrap().push(format!("embed: {}", text));
        Ok(Vec::new())
    }
}

#[async_trait]
impl EventBusConnector for NullConnector {
    async fn emit(&self, topic: &str, _payload: serde_json::Value) -> Result<()> {
        self.calls.lock().unwrap().push(format!("emit: {}", topic));
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("subscribe: {}", topic));
        Ok(())
    }
}
