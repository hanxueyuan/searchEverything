use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct SearchResult {
    pub path: String,
    pub size: u64,
    pub modified: String,
    pub is_dir: bool,
}

#[derive(Clone)]
pub enum OutputFormat {
    Text,
    Json,
}

impl SearchResult {
    pub fn format(&self, format: &OutputFormat) -> String {
        match format {
            OutputFormat::Text => self.path.clone(),
            OutputFormat::Json => serde_json::to_string(self).unwrap_or_default(),
        }
    }
}
