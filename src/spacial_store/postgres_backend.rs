use std::cell::RefCell;
use std::fs;

use anyhow::Result;
use postgres::{Client, NoTls};
use serde_json::Value;
use uuid::Uuid;
use std::fmt;

use crate::spacial_store::types::{Point, Region};
use crate::spacial_store::backend::PersistenceBackend;

/// Manages the connection to the Postgres database and provides methods for data manipulation.
pub struct PostgresDatabase {
    client: RefCell<Client>,
}

impl PostgresDatabase {
    /// Creates a new PostgresDatabase instance.
    ///
    /// # Arguments
    ///
    /// * `conn_str` - Postgres connection string (e.g., `"host=localhost user=postgres password=secret dbname=spatial"`).
    ///
    /// # Returns
    ///
    /// A Result containing a new PostgresDatabase instance or an error.
    ///
    /// # Examples
    ///
    /// ```
    /// let db = PostgresDatabase::new("host=localhost user=postgres password=foo dbname=mydb")
    ///     .expect("failed to connect to Postgres");
    /// ```
    pub fn new(conn_str: &str) -> Result<Self> {
        let client = Client::connect(conn_str, NoTls)?;
        Ok(PostgresDatabase {
            client: RefCell::new(client),
        })
    }

    /// Creates the necessary tables in Postgres if they don't exist.
    pub fn create_table(&self) -> Result<()> {
        let sql = "
            CREATE TABLE IF NOT EXISTS points (
                id TEXT PRIMARY KEY,
                x DOUBLE PRECISION NOT NULL,
                y DOUBLE PRECISION NOT NULL,
                z DOUBLE PRECISION NOT NULL,
                dataFile TEXT NOT NULL,
                region_id TEXT,
                object_type TEXT NOT NULL,
                sizeX DOUBLE PRECISION NOT NULL,
                sizeY DOUBLE PRECISION NOT NULL,
                sizeZ DOUBLE PRECISION NOT NULL
            );
            CREATE TABLE IF NOT EXISTS regions (
                id TEXT PRIMARY KEY,
                center_x DOUBLE PRECISION NOT NULL,
                center_y DOUBLE PRECISION NOT NULL,
                center_z DOUBLE PRECISION NOT NULL,
                size DOUBLE PRECISION NOT NULL
            );
        ";
        self.client.borrow_mut().batch_execute(sql)?;
        Ok(())
    }

    /// Adds or updates a point, storing its custom data in a file.
    pub fn add_point(&self, point: &Point, region_id: Uuid) -> Result<()> {
        let id = point.id.unwrap_or_else(Uuid::new_v4).to_string();
        let region_str = region_id.to_string();
        let custom_data_str = serde_json::to_string(&point.custom_data)?;
        let folder = &id[..2];
        let file_path = format!("./data/{}/{}", folder, id);

        fs::create_dir_all(format!("./data/{}", folder))?;
        fs::write(&file_path, &custom_data_str)?;

        let mut client = self.client.borrow_mut();
        client.execute(
            "\
            INSERT INTO points (id, x, y, z, dataFile, region_id, object_type, sizeX, sizeY, sizeZ)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            ON CONFLICT (id) DO UPDATE SET
              x = EXCLUDED.x,
              y = EXCLUDED.y,
              z = EXCLUDED.z,
              dataFile = EXCLUDED.dataFile,
              region_id = EXCLUDED.region_id,
              object_type = EXCLUDED.object_type,
              sizeX = EXCLUDED.sizeX,
              sizeY = EXCLUDED.sizeY,
              sizeZ = EXCLUDED.sizeZ",
            &[
                &id,
                &point.x,
                &point.y,
                &point.z,
                &file_path,
                &region_str,
                &point.object_type,
                &point.size_x,
                &point.size_y,
                &point.size_z,
            ],
        )?;
        Ok(())
    }

    /// Retrieves all points within a given radius of (x1,y1,z1).
    pub fn get_points_within_radius(
        &self,
        x1: f64,
        y1: f64,
        z1: f64,
        radius: f64,
    ) -> Result<Vec<Point>> {
        let radius_sq = radius * radius;
        let mut client = self.client.borrow_mut();
        let rows = client.query(
            "\
            SELECT id, x, y, z, dataFile, object_type, sizeX, sizeY, sizeZ
            FROM points
            WHERE ((x - $1)*(x - $1) + (y - $2)*(y - $2) + (z - $3)*(z - $3)) <= $4",
            &[&x1, &y1, &z1, &radius_sq],
        )?;

        let mut points = Vec::new();
        for row in rows {
            let id_str: String = row.get(0);
            let x: f64 = row.get(1);
            let y: f64 = row.get(2);
            let z: f64 = row.get(3);
            let data_file: String = row.get(4);
            let object_type: String = row.get(5);
            let size_x: f64 = row.get(6);
            let size_y: f64 = row.get(7);
            let size_z: f64 = row.get(8);

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

    /// Inserts or updates a region.
    pub fn create_region(&self, region_id: Uuid, center: [f64; 3], size: f64) -> Result<()> {
        let region_str = region_id.to_string();
        let mut client = self.client.borrow_mut();
        client.execute(
            "\
            INSERT INTO regions (id, center_x, center_y, center_z, size)
            VALUES ($1,$2,$3,$4,$5)
            ON CONFLICT (id) DO UPDATE SET
              center_x = EXCLUDED.center_x,
              center_y = EXCLUDED.center_y,
              center_z = EXCLUDED.center_z,
              size = EXCLUDED.size",
            &[
                &region_str,
                &center[0],
                &center[1],
                &center[2],
                &size,
            ],
        )?;
        Ok(())
    }

    /// Deletes a single point by UUID.
    pub fn remove_point(&self, point_id: Uuid) -> Result<()> {
        let pid = point_id.to_string();
        self.client
            .borrow_mut()
            .execute("DELETE FROM points WHERE id = $1", &[&pid])?;
        Ok(())
    }

    /// Updates the (x,y,z) of an existing point.
    pub fn update_point_position(&self, point_id: Uuid, x: f64, y: f64, z: f64) -> Result<()> {
        let pid = point_id.to_string();
        self.client.borrow_mut().execute(
            "UPDATE points SET x = $1, y = $2, z = $3 WHERE id = $4",
            &[&x, &y, &z, &pid],
        )?;
        Ok(())
    }

    /// Returns every region in the DB.
    pub fn get_all_regions(&self) -> Result<Vec<Region>> {
        let mut client = self.client.borrow_mut();
        let rows = client.query(
            "SELECT id, center_x, center_y, center_z, size FROM regions",
            &[],
        )?;

        let mut regions = Vec::new();
        for row in rows {
            let id_str: String = row.get(0);
            regions.push(Region {
                id: Uuid::parse_str(&id_str)?,
                center: [row.get(1), row.get(2), row.get(3)],
                size: row.get(4),
            });
        }
        Ok(regions)
    }

    /// Returns all points in a specific region.
    pub fn get_points_in_region(&self, region_id: Uuid) -> Result<Vec<Point>> {
        let region_str = region_id.to_string();
        let mut client = self.client.borrow_mut();
        let rows = client.query(
            "\
            SELECT id, x, y, z, dataFile, object_type, sizeX, sizeY, sizeZ
            FROM points
            WHERE region_id = $1",
            &[&region_str],
        )?;

        let mut points = Vec::new();
        for row in rows {
            let id_str: String = row.get(0);
            let x: f64 = row.get(1);
            let y: f64 = row.get(2);
            let z: f64 = row.get(3);
            let data_file: String = row.get(4);
            let object_type: String = row.get(5);
            let size_x: f64 = row.get(6);
            let size_y: f64 = row.get(7);
            let size_z: f64 = row.get(8);

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

    /// Deletes all points.
    pub fn clear_all_points(&self) -> Result<()> {
        self.client.borrow_mut().execute("DELETE FROM points", &[])?;
        Ok(())
    }
}

impl fmt::Debug for PostgresDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostgresDatabase")
            .field("client", &"[Client hidden]")
            .finish()
    }
}
impl PersistenceBackend for PostgresDatabase {
    fn create_table(&self) -> Result<()> {
        self.create_table()
    }

    fn add_point(&self, point: &Point, region_id: Uuid) -> Result<()> {
        self.add_point(point, region_id)
    }

    fn get_points_within_radius(
        &self,
        x: f64,
        y: f64,
        z: f64,
        radius: f64,
    ) -> Result<Vec<Point>> {
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
