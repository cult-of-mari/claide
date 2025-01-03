use serde::{Deserialize, Serialize};
use serde_json::Value as Object;

use self::content::Part;
use self::generation_config::GenerationConfig;
use self::safety_setting::SafetySetting;
use self::system_instructions::SystemInstructions;

pub mod content;
mod generation_config;
mod safety_setting;
pub mod schema;
mod system_instructions;

#[derive(Serialize)]
pub struct GenerateContent<'a> {
    #[serde(skip_serializing_if = "SystemInstructions::is_empty")]
    system_instruction: SystemInstructions<'a>,
    pub contents: Vec<GeminiMessage>,
    #[serde(rename = "safetySettings", skip_serializing_if = "Vec::is_empty")]
    safety_settings: Vec<SafetySetting>,
    #[serde(skip_serializing_if = "GenerationConfig::is_text")]
    generation_config: GenerationConfig,
}

impl<'a> GenerateContent<'a> {
    pub const fn new() -> Self {
        Self {
            system_instruction: SystemInstructions::new(),
            contents: Vec::new(),
            safety_settings: Vec::new(),
            generation_config: GenerationConfig::new(),
        }
    }

    pub const fn system_instructions(mut self, system_instructions: &'a str) -> Self {
        self.system_instruction = SystemInstructions::from(system_instructions);
        self
    }

    pub const fn json(mut self, json: bool) -> Self {
        self.generation_config = self.generation_config.json(json);
        self
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
    pub usage_metadata: GeminiUsageMetadata,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiUsageMetadata {
    pub prompt_token_count: u32,
    #[serde(default)]
    pub candidates_token_count: u32,
    pub total_token_count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeminiCandidate {
    pub content: GeminiMessage,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeminiMessage {
    pub role: GeminiRole,
    pub parts: Vec<Part>,
}

impl GeminiMessage {
    pub fn new(role: GeminiRole, parts: Vec<Part>) -> Self {
        Self { role, parts }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiRole {
    User,
    Model,
}

/// The JSON payload for the `create_file` method of `v1beta/files` API.
#[derive(Serialize)]
pub struct CreateFile<'a> {
    file: FileOptions<'a>,
}

#[derive(Serialize)]
struct FileOptions<'a> {
    display_name: &'a str,
}

impl<'a> CreateFile<'a> {
    pub const fn new(display_name: &'a str) -> Self {
        Self {
            file: FileOptions { display_name },
        }
    }
}

/// Response JSON from the `create_file` method.
#[derive(Deserialize)]
pub struct CreateFileResponse {
    pub file: File,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub name: String,
    pub display_name: String,
    pub mime_type: String,
    pub size_bytes: String,
    pub create_time: String,
    pub update_time: String,
    pub expiration_time: String,
    pub sha256_hash: String,
    pub uri: String,
    pub state: State,
    #[serde(default)]
    pub error: Option<Status>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum State {
    #[serde(rename = "PROCESSING")]
    Pending,
    #[serde(rename = "ACTIVE")]
    Ready,
    #[serde(rename = "FAILED")]
    Error,
}

#[derive(Deserialize)]
pub struct Status {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub details: Vec<Object>,
}

#[derive(Serialize)]
#[serde(transparent)]
pub struct Authentication<'a> {
    query_params: [Key<'a>; 1],
}

#[derive(Serialize)]
struct Key<'a> {
    key: &'a str,
}

impl<'a> Authentication<'a> {
    pub const fn new(api_key: &'a str) -> Self {
        Self {
            query_params: [Key { key: api_key }],
        }
    }
}
