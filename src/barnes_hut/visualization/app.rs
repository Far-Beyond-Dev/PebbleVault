use crate::BarnesHutManager;
use crate::structs::SpatialObject;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::input::{RenderArgs, UpdateArgs};
use uuid::Uuid;

const STAR_WIDTH: f64 = 2.0;
const BLACKHOLE_WIDTH: f64 = 18.0;
const EVENTHORIZON_WIDTH: f64 = 20.0;
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
const SOLID_BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const RED1: [f32; 4] = [1.0, 0.0, 0.0, 0.1];
const WHITE7: [f32; 4] = [1.0, 1.0, 1.0, 0.7];

pub struct App {
    gl: GlGraphics,
    manager: BarnesHutManager,
    region_id: Uuid,
}

impl App {
    pub fn new(opengl: OpenGL, manager: BarnesHutManager, region_id: Uuid) -> Self {
        App {
            gl: GlGraphics::new(opengl),
            manager,
            region_id,
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let region = self.manager.regions.get(&self.region_id).unwrap().lock().unwrap();
        let objects: Vec<SpatialObject> = region.rtree.iter().cloned().collect();

        self.gl.draw(args.viewport(), |c, gl| {
            clear(BLACK, gl);

            for obj in &objects {
                let (x, y, _) = (obj.point[0], obj.point[1], obj.point[2]);
                
                // Assume the first few objects are black holes (you may need to adjust this logic)
                if obj.uuid.as_u128() % 1000 == 0 {
                    let transform = c.transform
                        .trans(x, y)
                        .trans(-EVENTHORIZON_WIDTH / 2.0, -EVENTHORIZON_WIDTH / 2.0);
                    ellipse(RED1, rectangle::square(0.0, 0.0, EVENTHORIZON_WIDTH), transform, gl);
                    
                    let transform_center = c.transform
                        .trans(x, y)
                        .trans(-BLACKHOLE_WIDTH / 2.0, -BLACKHOLE_WIDTH / 2.0);
                    ellipse(SOLID_BLACK, rectangle::square(0.0, 0.0, BLACKHOLE_WIDTH), transform_center, gl);
                } else {
                    let transform = c.transform
                        .trans(x, y)
                        .trans(-STAR_WIDTH / 2.0, -STAR_WIDTH / 2.0);
                    ellipse(WHITE7, rectangle::square(0.0, 0.0, STAR_WIDTH), transform, gl);
                }
            }
        });
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        self.manager.step_simulation(self.region_id).unwrap();
    }

    pub fn click(&mut self, mouse_xy: [f64; 2]) {
        let (x, y) = (mouse_xy[0], mouse_xy[1]);
        println!("Clicked at ({}, {})", x, y);
        
        // Here you can add logic to create a new black hole or interact with the simulation
        // For example:
        // self.manager.add_object(self.region_id, Uuid::new_v4(), x, y, 0.0, "Black Hole").unwrap();
    }
}