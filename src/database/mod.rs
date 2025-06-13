use std::collections::HashMap;
use std::fs;
use std::io::Write;

use toml::Value;

const DATABASE_DIR: &str = ".ffs";
const DATABASE_FILENAME: &str = "database.toml";

pub struct Database {
    filename: String,
    data: HashMap<String, String>,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    /// Creates a new database instance and loads data from the file.
    ///
    /// # Panics
    ///
    /// Panics if the database cannot be loaded from the file system.
    #[must_use]
    pub fn new() -> Self {
        let mut db = Self {
            filename: DATABASE_FILENAME.to_string(),
            data: HashMap::new(),
        };
        db.load().unwrap();
        db
    }
}

impl Database {
    /// Loads the database from the file system.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be loaded or created.
    ///
    /// # Panics
    ///
    /// Panics if the HOME environment variable is not set.
    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let home = std::env::var("HOME").unwrap();
        let database_dir = format!("{home}/{DATABASE_DIR}");
        let database_file = format!("{database_dir}/{DATABASE_FILENAME}");
        // create the database directory if it doesn't exist
        if fs::metadata(&database_dir).is_err() {
            println!("Creating database directory: {database_dir}");
            fs::create_dir_all(database_dir)?;
        }
        // create the database file if it doesn't exist
        if fs::metadata(&database_file).is_err() {
            println!("Creating database file: {database_file}");
            fs::File::create(&database_file)?;
        }
        let contents = fs::read_to_string(&database_file)?;
        let value = contents.parse::<Value>()?;
        let table = value.as_table().unwrap();
        self.data = table
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
            .collect();
        Ok(())
    }

    /// Sets a key-value pair in the database.
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be written to the file.
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

    #[must_use]
    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
}
