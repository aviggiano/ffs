use std::collections::HashMap;
use std::fs;
use std::io::Write;

use toml::Value;

pub struct Database {
    filename: String,
    data: HashMap<String, String>,
}

impl Database {
    pub fn new() -> Self {
        let mut db = Self {
            filename: "database.toml".to_string(),
            data: HashMap::new(),
        };
        db.load().unwrap();
        db
    }
}

impl Database {
    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // parse toml as key-value store
        let contents = fs::read_to_string(&self.filename)?;
        let value = contents.parse::<Value>()?;
        let table = value.as_table().unwrap();
        self.data = table
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
            .collect();
        Ok(())
    }

    pub fn set(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.data.insert(key.to_string(), value.to_string());
        let mut file = fs::File::create(&self.filename)?;
        let toml = toml::to_string(&self.data)?;
        file.write_all(toml.as_bytes())?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
}
