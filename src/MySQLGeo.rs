use rusqlite::{params, Connection, Result as SqlResult};
use serde_json::{self, Value};
use serde::{Serialize, Deserialize};


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    id: Option<i32>,
    x: f64,
    y: f64,
    z: f64,
    data: Value,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64, data: Value) -> Self {
        Point { id: None, x, y, z, data }
    }
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Database { conn })
    }

    pub fn create_table(&self) -> SqlResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS points (
                id INTEGER PRIMARY KEY,
                x REAL NOT NULL,
                y REAL NOT NULL,
                z REAL NOT NULL,
                data TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn add_point(&self, point: &Point) -> SqlResult<()> {
        let data_str = serde_json::to_string(&point.data)
            .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;
        
        self.conn.execute(
            "INSERT INTO points (x, y, z, data) VALUES (?1, ?2, ?3, ?4)",
            params![point.x, point.y, point.z, data_str],
        )?;
        
        Ok(())
    }

    pub fn get_points_within_radius(&self, x1: f64, y1: f64, z1: f64, radius: f64) -> SqlResult<Vec<Point>> {
        let radius_sq = radius * radius;
        let mut stmt = self.conn.prepare(
            "SELECT id, x, y, z, data FROM points
             WHERE ((x - ?1) * (x - ?1) + (y - ?2) * (y - ?2) + (z - ?3) * (z - ?3)) <= ?4",
        )?;
        
        let points_iter = stmt.query_map(params![x1, y1, z1, radius_sq], |row| {
            let id: i32 = row.get(0)?;
            let x: f64 = row.get(1)?;
            let y: f64 = row.get(2)?;
            let z: f64 = row.get(3)?;
            let data_str: String = row.get(4)?;
            let data: Value = serde_json::from_str(&data_str).unwrap();
            Ok(Point {
                id: Some(id),
                x,
                y,
                z,
                data,
            })
        })?;
        
        let mut points = Vec::new();
        for point in points_iter {
            points.push(point?);
        }
        
        Ok(points)
    }
    
}
