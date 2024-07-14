use rusqlite::{params, Connection, Result, Result as SqlResult};
use serde_json::Value;
use std::sync::{Arc, Mutex};

pub struct Vault {
    connection: Connection,
}

impl Vault {
    pub fn new(db_path: &str) -> Self {
        let connection = Connection::open(db_path).unwrap();
        Vault { connection }
    }

    pub fn define_class(&self, class_name: &str, schema: &str) -> SqlResult<()> {
        let schema: Value = serde_json::from_str(schema)?;
        let mut columns = Vec::new();
        let mut join_tables = Vec::new();

        if let Some(properties) = schema.as_object() {
            for (field_name, field_type) in properties {
                if field_type == "array" {
                    let join_table = format!("{}_{}_join", class_name, field_name);
                    join_tables.push((join_table.clone(), field_name.clone()));
                    self.connection.execute(
                        &format!(
                            "CREATE TABLE IF NOT EXISTS {} ({}_id INTEGER, {}_id INTEGER)",
                            join_table, class_name, field_name
                        ),
                        [],
                    )?;
                } else {
                    let sql_type = match field_type.as_str().unwrap() {
                        "string" => "TEXT",
                        "int" => "INTEGER",
                        _ => "TEXT",
                    };
                    columns.push(format!("{} {}", field_name, sql_type));
                }
            }
        }

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY AUTOINCREMENT, {})",
            class_name,
            columns.join(", ")
        );
        self.connection.execute(&sql, [])?;
        Ok(())
    }

    pub fn collect(&self, class_name: &str, data: &str) -> SqlResult<()> {
        let data: Value = serde_json::from_str(data)?;
        let mut columns = Vec::new();
        let mut values = Vec::new();
        let mut join_entries = Vec::new();

        if let Some(properties) = data.as_object() {
            for (field_name, field_value) in properties {
                if field_value.is_array() {
                    let join_table = format!("{}_{}_join", class_name, field_name);
                    if let Some(array) = field_value.as_array() {
                        for value in array {
                            join_entries.push((join_table.clone(), value.clone()));
                        }
                    }
                } else {
                    columns.push(field_name.clone());
                    values.push(format!("'{}'", field_value));
                }
            }
        }

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            class_name,
            columns.join(", "),
            values.join(", ")
        );

        self.connection.execute(&sql, [])?;
        let last_id = self.connection.last_insert_rowid();
        for (join_table, value) in join_entries {
            let sql = format!(
                "INSERT INTO {} ({}_id, {}_id) VALUES (?, ?)",
                join_table, class_name, "ingredient"
            );
            self.connection.execute(&sql, params![last_id, value.as_i64().unwrap()])?;
        }

        Ok(())
    }

    pub fn throw(&self, class_name: &str, id: i32) -> Result<()> {
        let sql = format!("DELETE FROM {} WHERE id = ?", class_name);
        self.connection.execute(&sql, params![id])?;
        Ok(())
    }

    pub fn drop(&self, class_name: &str) -> Result<()> {
        let sql = format!("DROP TABLE IF EXISTS {}", class_name);
        self.connection.execute(&sql, [])?;
        Ok(())
    }

    pub fn skim(&self, class_name: &str, id: i32) -> Result<Option<Value>> {
        let sql = format!("SELECT * FROM {} WHERE id = ?", class_name);
        let mut stmt = self.connection.prepare(&sql)?;

        let column_names: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|&name| name.to_string())
            .collect();

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let mut result = serde_json::Map::new();
            for (i, column_name) in column_names.iter().enumerate() {
                result.insert(column_name.clone(), Value::String(row.get::<usize, String>(i)?));
            }
            return Ok(Some(Value::Object(result)));
        }

        Ok(None)
    }

    pub fn pebbleshift(&self, class_name: &str, id: i32, new_data: &str) -> Result<()> {
        let new_data: Value = serde_json::from_str(new_data).unwrap();
        let mut sets = Vec::new();

        if let Some(properties) = new_data.as_object() {
            for (field_name, field_value) in properties {
                sets.push(format!("{} = '{}'", field_name, field_value));
            }
        }

        let sql = format!(
            "UPDATE {} SET {} WHERE id = ?",
            class_name,
            sets.join(", ")
        );

        self.connection.execute(&sql, params![id])?;
        Ok(())
    }

    pub fn pebblesift(&self, class_name: &str, query_conditions: &str) -> Result<Vec<Value>> {
        let sql = format!("SELECT * FROM {} WHERE {}", class_name, query_conditions);
        let mut stmt = self.connection.prepare(&sql)?;

        let column_names: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|&name| name.to_string())
            .collect();

        let rows = stmt.query_map([], |row| {
            let mut result = serde_json::Map::new();
            for (i, column_name) in column_names.iter().enumerate() {
                result.insert(column_name.clone(), Value::String(row.get::<usize, String>(i)?));
            }
            Ok(Value::Object(result))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}

pub fn main() {
    let db_path = "game_data.db"; // Adjust this path to your desired database file location
    let vault = Arc::new(Mutex::new(Vault::new(db_path)));

    {
        let vault = vault.lock().unwrap();
        vault.define_class("ingredient", r#"{
            "name": "string"
        }"#).unwrap();

        vault.define_class("recipe", r#"{
            "name": "string",
            "ingredients": "array",
            "outcome": "string",
            "base_craft_time": "int"
        }"#).unwrap();

        vault.collect("ingredient", r#"{"name": "sugar"}"#).unwrap();
        vault.collect("ingredient", r#"{"name": "flour"}"#).unwrap();
        vault.collect("recipe", r#"{"name": "cake", "ingredients": "[1, 2]", "outcome": "cake", "base_craft_time": 30}"#).unwrap();
    }
}