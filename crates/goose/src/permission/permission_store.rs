use crate::message::ToolRequest;
use anyhow::Result;
use blake3::Hasher;
use chrono::Utc;
use etcetera::{choose_app_strategy, AppStrategy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use std::{fs::File, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolPermissionRecord {
    tool_name: String,
    allowed: bool,
    context_hash: String, // Hash of the tool's arguments/context to differentiate similar calls
    #[serde(skip_serializing_if = "Option::is_none")] // Don't serialize if None
    readable_context: Option<String>, // Add this field
    timestamp: i64,
    expiry: Option<i64>, // Optional expiry timestamp
}

#[derive(Debug, Serialize, Deserialize)]
enum StorageType {
    #[serde(skip)]
    Memory,
    #[serde(skip)]
    File { permissions_dir: PathBuf },
}

impl Default for StorageType {
    fn default() -> Self {
        StorageType::File { 
            permissions_dir: PathBuf::from(".config/goose") 
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolPermissionStore {
    permissions: HashMap<String, Vec<ToolPermissionRecord>>,
    version: u32, // For future schema migrations
    #[serde(skip)] // Don't serialize this field
    storage: StorageType,
}

impl Default for ToolPermissionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolPermissionStore {
    pub fn new() -> Self {
        // Check if we should use in-memory storage
        if std::env::var("GOOSE_IN_MEMORY_CONFIG").is_ok() {
            return Self {
                permissions: HashMap::new(),
                version: 1,
                storage: StorageType::Memory,
            };
        }
        
        let permissions_dir = choose_app_strategy(crate::config::APP_STRATEGY.clone())
            .map(|strategy| strategy.config_dir())
            .unwrap_or_else(|_| PathBuf::from(".config/goose"));

        Self {
            permissions: HashMap::new(),
            version: 1,
            storage: StorageType::File { permissions_dir },
        }
    }
    
    pub fn new_in_memory() -> Self {
        Self {
            permissions: HashMap::new(),
            version: 1,
            storage: StorageType::Memory,
        }
    }

    pub fn load() -> Result<Self> {
        let store = Self::new();
        
        // If using in-memory storage, just return the empty store
        if matches!(store.storage, StorageType::Memory) {
            return Ok(store);
        }
        
        // Get the permissions directory from the File storage
        let permissions_dir = match &store.storage {
            StorageType::File { permissions_dir } => permissions_dir.clone(),
            _ => unreachable!(), // We already checked for Memory above
        };
        
        let file_path = permissions_dir.join("tool_permissions.json");

        if !file_path.exists() {
            return Ok(store);
        }

        let file = File::open(file_path)?;
        let mut permissions: ToolPermissionStore = serde_json::from_reader(file)?;
        
        // Update the storage type to match the original store
        permissions.storage = store.storage;

        // Clean up expired entries on load
        permissions.cleanup_expired()?;

        Ok(permissions)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        // If using in-memory storage, we don't need to save to disk
        if matches!(self.storage, StorageType::Memory) {
            return Ok(());
        }
        
        // Get the permissions directory from the File storage
        let permissions_dir = match &self.storage {
            StorageType::File { permissions_dir } => permissions_dir,
            _ => unreachable!(), // We already checked for Memory above
        };
        
        std::fs::create_dir_all(permissions_dir)?;

        let path = permissions_dir.join("tool_permissions.json");
        let temp_path = path.with_extension("tmp");

        // Write complete content to temporary file
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&temp_path, &content)?;

        // Atomically rename temp file to target file
        std::fs::rename(temp_path, path)?;

        Ok(())
    }

    pub fn check_permission(&self, tool_request: &ToolRequest) -> Option<bool> {
        let context_hash = self.hash_tool_context(tool_request);
        let tool_call = tool_request.tool_call.as_ref().unwrap();
        let key = format!("{}:{}", tool_call.name, context_hash);

        self.permissions.get(&key).and_then(|records| {
            records
                .iter()
                .filter(|record| record.expiry.is_none_or(|exp| exp > Utc::now().timestamp()))
                .next_back()
                .map(|record| record.allowed)
        })
    }

    pub fn record_permission(
        &mut self,
        tool_request: &ToolRequest,
        allowed: bool,
        expiry_duration: Option<Duration>,
    ) -> anyhow::Result<()> {
        let context_hash = self.hash_tool_context(tool_request);
        let tool_call = tool_request.tool_call.as_ref().unwrap();
        let key = format!("{}:{}", tool_call.name, context_hash);

        let record = ToolPermissionRecord {
            tool_name: tool_call.name.clone(),
            allowed,
            context_hash,
            readable_context: Some(tool_request.to_readable_string()),
            timestamp: Utc::now().timestamp(),
            expiry: expiry_duration.map(|d| Utc::now().timestamp() + d.as_secs() as i64),
        };

        self.permissions.entry(key).or_default().push(record);

        self.save()?;
        Ok(())
    }

    fn hash_tool_context(&self, tool_request: &ToolRequest) -> String {
        // Create a hash of the tool's arguments to differentiate similar calls
        // This helps identify when the same tool is being used in a different context
        let mut hasher = Hasher::new();
        hasher.update(
            serde_json::to_string(&tool_request.tool_call.as_ref().unwrap().arguments)
                .unwrap_or_default()
                .as_bytes(),
        );
        hasher.finalize().to_hex().to_string()
    }

    pub fn cleanup_expired(&mut self) -> anyhow::Result<()> {
        let now = Utc::now().timestamp();
        let mut changed = false;

        self.permissions.retain(|_, records| {
            records.retain(|record| record.expiry.is_none_or(|exp| exp > now));
            changed = changed || records.is_empty();
            !records.is_empty()
        });

        if changed {
            self.save()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_in_memory_permission_store() -> anyhow::Result<()> {
        // Create an in-memory store
        let mut store = ToolPermissionStore::new_in_memory();
        
        // Create a mock tool call
        let tool_call = mcp_core::tool::ToolCall {
            name: "test_tool".to_string(),
            arguments: serde_json::json!({"key": "value"}),
        };
        
        // Create a mock tool request
        let tool_request = ToolRequest {
            id: "test_id".to_string(),
            tool_call: Ok(tool_call),
        };
        
        // Record a permission
        store.record_permission(&tool_request, true, None)?;
        
        // Check if the permission was recorded
        let permission = store.check_permission(&tool_request);
        assert_eq!(permission, Some(true));
        
        Ok(())
    }
    
    #[test]
    fn test_env_var_in_memory_permission_store() -> anyhow::Result<()> {
        // Set the environment variable
        env::set_var("GOOSE_IN_MEMORY_CONFIG", "1");
        
        // Create a store - should be in-memory due to env var
        let mut store = ToolPermissionStore::new();
        
        // Create a mock tool call
        let tool_call = mcp_core::tool::ToolCall {
            name: "test_tool".to_string(),
            arguments: serde_json::json!({"key": "value"}),
        };
        
        // Create a mock tool request
        let tool_request = ToolRequest {
            id: "test_id".to_string(),
            tool_call: Ok(tool_call),
        };
        
        // Record a permission
        store.record_permission(&tool_request, true, None)?;
        
        // Check if the permission was recorded
        let permission = store.check_permission(&tool_request);
        assert_eq!(permission, Some(true));
        
        // Clean up
        env::remove_var("GOOSE_IN_MEMORY_CONFIG");
        
        Ok(())
    }
}
