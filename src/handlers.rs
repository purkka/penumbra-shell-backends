use serde_json::Value;

pub trait EventHandler: Send + Sync {
    fn handle_event(&self, event: &Value);
}
