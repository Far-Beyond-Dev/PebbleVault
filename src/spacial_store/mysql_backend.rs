use std::fs;
use std::sync::Mutex;

use anyhow::Result;
use mysql::{params, prelude::*, Pool, PooledConn, Row};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

use crate::spacial_store::backend::PersistenceBackend;
use crate::spacial_store::types::{Point, Region};

pub struct MySqlDatabase {
    conn: Mutex<PooledConn>,
}

impl MySqlDatabase {
    pub fn new(url: &str) -> Result<Self> {
        let pool = Pool::new(url)?;
        let conn = pool.get_conn()?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn create_table(&self) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        conn.query_drop(
            r"
            CREATE TABLE IF NOT EXISTS points (
                id VARCHAR(36) PRIMARY KEY,
                x DOUBLE NOT NULL,
                y DOUBLE NOT NULL,
                z DOUBLE NOT NULL,
                dataFile TEXT NOT NULL,
                region_id VARCHAR(36),
                object_type TEXT NOT NULL,
                sizeX DOUBLE NOT NULL,
                sizeY DOUBLE NOT NULL,
                sizeZ DOUBLE NOT NULL
            );
            CREATE TABLE IF NOT EXISTS regions (
                id VARCHAR(36) PRIMARY KEY,
                center_x DOUBLE NOT NULL,
                center_y DOUBLE NOT NULL,
                center_z DOUBLE NOT NULL,
                size DOUBLE NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn add_point(&self, point: &Point, region_id: Uuid) -> Result<()> {
        let id = point.id.unwrap_or_else(Uuid::new_v4).to_string();
        let custom_data_str = serde_json::to_string(&point.custom_data)?;
        let folder = &id[..2];
        let file_path = format!("./data/{}/{}", folder, id);
        fs::create_dir_all(format!("./data/{}", folder))?;
        fs::write(&file_path, &custom_data_str)?;

        let mut conn = self.conn.lock().unwrap();
        conn.exec_drop(
            r"INSERT INTO points (id, x, y, z, dataFile, region_id, object_type, sizeX, sizeY, sizeZ)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                x = VALUES(x), y = VALUES(y), z = VALUES(z),
                dataFile = VALUES(dataFile), region_id = VALUES(region_id),
                object_type = VALUES(object_type), sizeX = VALUES(sizeX),
                sizeY = VALUES(sizeY), sizeZ = VALUES(sizeZ)",
            (
                &id,
                point.x,
                point.y,
                point.z,
                &file_path,
                region_id.to_string(),
                &point.object_type,
                point.size_x,
                point.size_y,
                point.size_z,
            ),
        )?;
        Ok(())
    }

    pub fn get_points_within_radius(
        &self,
        x: f64,
        y: f64,
        z: f64,
        radius: f64,
    ) -> Result<Vec<Point>> {
        let radius_sq = radius * radius;
        let mut conn = self.conn.lock().unwrap();
        let rows: Vec<Row> = conn.exec(
            r"SELECT id, x, y, z, dataFile, object_type, sizeX, sizeY, sizeZ
              FROM points
              WHERE POW(x - :x, 2) + POW(y - :y, 2) + POW(z - :z, 2) <= :radius_sq",
            params! {
                "x" => x,
                "y" => y,
                "z" => z,
                "radius_sq" => radius_sq,
            },
        )?;

        let mut points = Vec::new();
        for row in rows {
            let (id_str, x, y, z, data_file, object_type, size_x, size_y, size_z): (
                String,
                f64,
                f64,
                f64,
                String,
                String,
                f64,
                f64,
                f64,
            ) = mysql::from_row(row);
            let data = fs::read_to_string(&data_file)?;
            let custom_data: Value = serde_json::from_str(&data)?;
            points.push(Point {
                id: Some(Uuid::parse_str(&id_str)?),
                x,
                y,
                z,
                size_x,
                size_y,
                size_z,
                object_type,
                custom_data,
            });
        }

        Ok(points)
    }

    pub fn create_region(&self, region_id: Uuid, center: [f64; 3], size: f64) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        conn.exec_drop(
            r"INSERT INTO regions (id, center_x, center_y, center_z, size)
              VALUES (:id, :x, :y, :z, :size)
              ON DUPLICATE KEY UPDATE
                center_x = VALUES(center_x), center_y = VALUES(center_y),
                center_z = VALUES(center_z), size = VALUES(size)",
            params! {
                "id" => region_id.to_string(),
                "x" => center[0],
                "y" => center[1],
                "z" => center[2],
                "size" => size,
            },
        )?;
        Ok(())
    }

    pub fn remove_point(&self, point_id: Uuid) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        conn.exec_drop(
            "DELETE FROM points WHERE id = :id",
            params! {
                "id" => point_id.to_string(),
            },
        )?;
        Ok(())
    }

    pub fn update_point_position(&self, point_id: Uuid, x: f64, y: f64, z: f64) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        conn.exec_drop(
            "UPDATE points SET x = :x, y = :y, z = :z WHERE id = :id",
            params! {
                "x" => x,
                "y" => y,
                "z" => z,
                "id" => point_id.to_string(),
            },
        )?;
        Ok(())
    }

    pub fn get_all_regions(&self) -> Result<Vec<Region>> {
        let mut conn = self.conn.lock().unwrap();
        let rows: Vec<Row> =
            conn.query("SELECT id, center_x, center_y, center_z, size FROM regions")?;
        let mut regions = Vec::new();
        for row in rows {
            let (id_str, x, y, z, size): (String, f64, f64, f64, f64) = mysql::from_row(row);
            regions.push(Region {
                id: Uuid::parse_str(&id_str)?,
                center: [x, y, z],
                size,
            });
        }
        Ok(regions)
    }

    pub fn get_points_in_region(&self, region_id: Uuid) -> Result<Vec<Point>> {
        let mut conn = self.conn.lock().unwrap();
        let rows: Vec<Row> = conn.exec(
            r"SELECT id, x, y, z, dataFile, object_type, sizeX, sizeY, sizeZ
              FROM points WHERE region_id = :region_id",
            params! {
                "region_id" => region_id.to_string(),
            },
        )?;

        let mut points = Vec::new();
        for row in rows {
            let (id_str, x, y, z, data_file, object_type, size_x, size_y, size_z): (
                String,
                f64,
                f64,
                f64,
                String,
                String,
                f64,
                f64,
                f64,
            ) = mysql::from_row(row);
            let data = fs::read_to_string(&data_file)?;
            let custom_data: Value = serde_json::from_str(&data)?;
            points.push(Point {
                id: Some(Uuid::parse_str(&id_str)?),
                x,
                y,
                z,
                size_x,
                size_y,
                size_z,
                object_type,
                custom_data,
            });
        }

        Ok(points)
    }

    pub fn clear_all_points(&self) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        conn.query_drop("DELETE FROM points")?;
        Ok(())
    }
}

impl fmt::Debug for MySqlDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySqlDatabase")
            .field("conn", &"[PooledConn hidden]")
            .finish()
    }
}

impl PersistenceBackend for MySqlDatabase {
    fn create_table(&self) -> Result<()> {
        self.create_table()
    }

    fn add_point(&self, point: &Point, region_id: Uuid) -> Result<()> {
        self.add_point(point, region_id)
    }

    fn get_points_within_radius(&self, x: f64, y: f64, z: f64, radius: f64) -> Result<Vec<Point>> {
        self.get_points_within_radius(x, y, z, radius)
    }

    fn create_region(&self, region_id: Uuid, center: [f64; 3], size: f64) -> Result<()> {
        self.create_region(region_id, center, size)
    }

    fn remove_point(&self, point_id: Uuid) -> Result<()> {
        self.remove_point(point_id)
    }

    fn update_point_position(&self, point_id: Uuid, x: f64, y: f64, z: f64) -> Result<()> {
        self.update_point_position(point_id, x, y, z)
    }

    fn get_all_regions(&self) -> Result<Vec<Region>> {
        self.get_all_regions()
    }

    fn get_points_in_region(&self, region_id: Uuid) -> Result<Vec<Point>> {
        self.get_points_in_region(region_id)
    }

    fn clear_all_points(&self) -> Result<()> {
        self.clear_all_points()
    }
}
