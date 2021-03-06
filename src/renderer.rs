
use std::vec::*;
use graphics::*;
use math::*;

use na::{Vector3};

pub trait RendererVertFrag{
    fn render_mode       (&self) -> usize;
    fn shader_name       (&self) -> String;
    fn set_attrib_ptrs   (&mut self);
    fn construct         (&mut self) -> bool;
    fn deconstruct       (&mut self) -> bool;
    fn draw              (&mut self) -> bool;
    fn reset             (&mut self); //used to clear/reset all data stored in 'self'
}

pub struct RendererVertFragDef{
    pub vertex_size: usize,
    pub vertex_pool: Vec<f32>,
    pub index_pool: Vec<u32>,
    pub vertex_count: u32,
    pub vbo: usize,
    pub vao: usize,
    pub ebo: usize,
    pub constructed: bool,
    pub render_mode: usize,
    pub shader_name: String,
    pub set_attrib_ptrs: fn(&mut RendererVertFragDef),

}


pub const VERTEX_SIZE_COLOR : usize = 6;
pub const VERTEX_SIZE_COLOR_NORMAL : usize = 9;

pub fn set_attrib_ptrs_color(_:&mut RendererVertFragDef){
    gl_vertex_attrib_pointer(0, 3, GL_FLOAT, false, VERTEX_SIZE_COLOR * 4,
                             0);
    gl_enable_vertex_attrib_array(0);

    gl_vertex_attrib_pointer(1, 3, GL_FLOAT, false, VERTEX_SIZE_COLOR * 4,
                             3 * 4);
    gl_enable_vertex_attrib_array(1);

}

pub fn set_attrib_ptrs_color_normal(_:&mut RendererVertFragDef){
    gl_vertex_attrib_pointer(0, 3, GL_FLOAT, false, VERTEX_SIZE_COLOR_NORMAL * 4,
                             0);
    gl_enable_vertex_attrib_array(0);

    gl_vertex_attrib_pointer(1, 3, GL_FLOAT, false, VERTEX_SIZE_COLOR_NORMAL * 4,
                             3 * 4);
    gl_enable_vertex_attrib_array(1);

    gl_vertex_attrib_pointer(2, 3, GL_FLOAT, false, VERTEX_SIZE_COLOR_NORMAL * 4,
                             6 * 4);
    gl_enable_vertex_attrib_array(2);

}

impl RendererVertFrag for RendererVertFragDef{
    fn render_mode(&self) -> usize {
        self.render_mode
    }

    fn shader_name(&self) -> String {
        self.shader_name.clone()
    }

    fn set_attrib_ptrs(&mut self) {
        (self.set_attrib_ptrs)(self)
    }

    fn construct(&mut self) -> bool {
        if self.constructed {return false;};

        self.vao = gl_gen_vertex_arrays();
        self.vbo = gl_gen_buffers();
        self.ebo = gl_gen_buffers();


        gl_bind_vertex_array(self.vao);

        gl_bind_buffer(GL_ARRAY_BUFFER, self.vbo);

        gl_buffer_data(GL_ARRAY_BUFFER,
                       self.vertex_pool.len(),
                       self.vertex_pool.as_slice(),
                       GL_STATIC_DRAW);

        gl_bind_buffer(GL_ELEMENT_ARRAY_BUFFER, self.ebo);
        gl_buffer_data(GL_ELEMENT_ARRAY_BUFFER, self.index_pool.len(),
                       self.index_pool.as_slice(),
                       GL_STATIC_DRAW
        );

        self.set_attrib_ptrs();

        gl_bind_buffer(GL_ARRAY_BUFFER, 0);
        gl_bind_vertex_array(0);

        self.constructed = true;

        true
    }

    fn deconstruct(&mut self) -> bool {
        if !self.constructed {return false;};

        gl_delete_vertex_arrays(self.vao);
        gl_delete_buffers(self.vbo);
        gl_delete_buffers(self.ebo);

        self.constructed = false;

        true
    }

    fn draw(&mut self) -> bool {
        if !self.constructed {return false;};

        gl_bind_vertex_array(self.vao);
        gl_draw_elements(self.render_mode, self.index_pool.len(), GL_UNSIGNED_INT, 0);
        gl_bind_vertex_array(0);

        true
    }

    fn reset(&mut self) {
        self.vertex_pool.clear();
        self.index_pool.clear();
        self.vertex_count = 0;
    }
}

impl RendererVertFragDef{
    pub fn make(vs: usize,
            set_attrib_ptrs : fn (&mut RendererVertFragDef),
            render_mode: usize,
            shader_name: String) -> RendererVertFragDef{
        RendererVertFragDef{
            vertex_size: vs,
            vertex_pool: Vec::new(),
            index_pool: Vec::new(),
            vertex_count: 0,
            vbo: 0,
            vao: 0,
            ebo: 0,
            constructed:false,
            render_mode,
            shader_name,
            set_attrib_ptrs
        }
    }
}

pub fn add_triangle_color(dat: &mut RendererVertFragDef, tr: &Triangle3<f32>, color: Vector3<f32>){
    dat.vertex_pool.push(tr.p1[0]);
    dat.vertex_pool.push(tr.p1[1]);
    dat.vertex_pool.push(tr.p1[2]);

    dat.vertex_pool.push(color[0]);
    dat.vertex_pool.push(color[1]);
    dat.vertex_pool.push(color[2]);

    dat.vertex_pool.push(tr.p2[0]);
    dat.vertex_pool.push(tr.p2[1]);
    dat.vertex_pool.push(tr.p2[2]);

    dat.vertex_pool.push(color[0]);
    dat.vertex_pool.push(color[1]);
    dat.vertex_pool.push(color[2]);

    dat.vertex_pool.push(tr.p3[0]);
    dat.vertex_pool.push(tr.p3[1]);
    dat.vertex_pool.push(tr.p3[2]);

    dat.vertex_pool.push(color[0]);
    dat.vertex_pool.push(color[1]);
    dat.vertex_pool.push(color[2]);

    dat.index_pool.push(dat.vertex_count + 0);
    dat.index_pool.push(dat.vertex_count + 1);
    dat.index_pool.push(dat.vertex_count + 2);


    dat.vertex_count += 3;
}


pub fn add_triangle_color_normal(dat: &mut RendererVertFragDef, tr: &Triangle3<f32>, color: &Vector3<f32>, normal : &Vector3<f32>){
    dat.vertex_pool.push(tr.p1[0]);
    dat.vertex_pool.push(tr.p1[1]);
    dat.vertex_pool.push(tr.p1[2]);

    dat.vertex_pool.push(color[0]);
    dat.vertex_pool.push(color[1]);
    dat.vertex_pool.push(color[2]);

    dat.vertex_pool.push(normal[0]);
    dat.vertex_pool.push(normal[1]);
    dat.vertex_pool.push(normal[2]);

    dat.vertex_pool.push(tr.p2[0]);
    dat.vertex_pool.push(tr.p2[1]);
    dat.vertex_pool.push(tr.p2[2]);

    dat.vertex_pool.push(color[0]);
    dat.vertex_pool.push(color[1]);
    dat.vertex_pool.push(color[2]);

    dat.vertex_pool.push(normal[0]);
    dat.vertex_pool.push(normal[1]);
    dat.vertex_pool.push(normal[2]);

    dat.vertex_pool.push(tr.p3[0]);
    dat.vertex_pool.push(tr.p3[1]);
    dat.vertex_pool.push(tr.p3[2]);

    dat.vertex_pool.push(color[0]);
    dat.vertex_pool.push(color[1]);
    dat.vertex_pool.push(color[2]);

    dat.vertex_pool.push(normal[0]);
    dat.vertex_pool.push(normal[1]);
    dat.vertex_pool.push(normal[2]);

    dat.index_pool.push(dat.vertex_count + 0);
    dat.index_pool.push(dat.vertex_count + 1);
    dat.index_pool.push(dat.vertex_count + 2);


    dat.vertex_count += 3;
}

fn add_vector_to_pool(dat : &mut RendererVertFragDef, vec : Vector3<f32>){
    for i in vec.iter(){dat.vertex_pool.push(i.clone());}
}

pub fn add_line3_color(dat : &mut RendererVertFragDef, line : Line3<f32>, color : Vector3<f32>){
    add_vector_to_pool(dat, line.start);
    add_vector_to_pool(dat, color);
    add_vector_to_pool(dat, line.end);
    add_vector_to_pool(dat, color);

    dat.index_pool.push(0 + dat.vertex_count);
    dat.index_pool.push(1 + dat.vertex_count);

    dat.vertex_count += 2;
}

pub fn add_square3_bounds_color(dat : &mut RendererVertFragDef, cube : Square3<f32>, color : Vector3<f32>){
    add_vector_to_pool(dat, Vector3::new(cube.center.x - cube.extent, cube.center.y - cube.extent, cube.center.z - cube.extent));
    add_vector_to_pool(dat, color);
    add_vector_to_pool(dat, Vector3::new(cube.center.x + cube.extent, cube.center.y - cube.extent, cube.center.z - cube.extent));
    add_vector_to_pool(dat, color);
    add_vector_to_pool(dat, Vector3::new(cube.center.x + cube.extent, cube.center.y + cube.extent, cube.center.z - cube.extent));
    add_vector_to_pool(dat, color);
    add_vector_to_pool(dat, Vector3::new(cube.center.x - cube.extent, cube.center.y + cube.extent, cube.center.z - cube.extent));
    add_vector_to_pool(dat, color);
    add_vector_to_pool(dat, Vector3::new(cube.center.x - cube.extent, cube.center.y - cube.extent, cube.center.z + cube.extent));
    add_vector_to_pool(dat, color);
    add_vector_to_pool(dat, Vector3::new(cube.center.x + cube.extent, cube.center.y - cube.extent, cube.center.z + cube.extent));
    add_vector_to_pool(dat, color);
    add_vector_to_pool(dat, Vector3::new(cube.center.x + cube.extent, cube.center.y + cube.extent, cube.center.z + cube.extent));
    add_vector_to_pool(dat, color);
    add_vector_to_pool(dat, Vector3::new(cube.center.x - cube.extent, cube.center.y + cube.extent, cube.center.z + cube.extent));
    add_vector_to_pool(dat, color);

    let indices : [u32;24] = [0,1,1,2,2,3,3,0, 4,5,5,6,6,7,7,4, 0,4, 1,5, 2,6, 3,7];
    for i in indices.iter() {dat.index_pool.push(i.clone() + dat.vertex_count);}
    dat.vertex_count += 8;
}


//for cubes
fn centers() -> [Vector3<f32>;8]{
    [Vector3::new(-0.5, -0.5, -0.5),
     Vector3::new(0.5, -0.5, -0.5),
     Vector3::new(0.5, -0.5, 0.5),
     Vector3::new(-0.5, -0.5, 0.5),

     Vector3::new(-0.5, 0.5, -0.5),
     Vector3::new(0.5, 0.5, -0.5),
     Vector3::new(0.5, 0.5, 0.5),
     Vector3::new(-0.5, 0.5, 0.5)]
}

pub fn add_cube_color_normal(dat : &mut RendererVertFragDef, cube : Square3<f32>, color : Vector3<f32>){
    let mut corners = [Vector3::zeros();8];

    for i in 0..8{
        corners[i] = centers()[i] * 2.0 * cube.extent + cube.center;
    }

    add_triangle_color_normal(dat, &Triangle3{p1 : corners[7], p2 : corners[0], p3 : corners[3]}, &color, &Vector3::new(-1.0, 0.0, 0.0));
    add_triangle_color_normal(dat, &Triangle3{p1 : corners[0], p2 : corners[7], p3 : corners[4]}, &color, &Vector3::new(-1.0, 0.0, 0.0));

    add_triangle_color_normal(dat, &Triangle3{p1 : corners[1], p2 : corners[6], p3 : corners[2]}, &color, &Vector3::new(1.0, 0.0, 0.0));
    add_triangle_color_normal(dat, &Triangle3{p1 : corners[1], p2 : corners[5], p3 : corners[6]}, &color, &Vector3::new(1.0, 0.0, 0.0));

    add_triangle_color_normal(dat, &Triangle3{p1 : corners[0], p2 : corners[4], p3 : corners[1]}, &color, &Vector3::new(0.0, 0.0, -1.0));
    add_triangle_color_normal(dat, &Triangle3{p1 : corners[1], p2 : corners[4], p3 : corners[5]}, &color, &Vector3::new(0.0, 0.0, -1.0));

    add_triangle_color_normal(dat, &Triangle3{p1 : corners[2], p2 : corners[7], p3 : corners[3]}, &color, &Vector3::new(0.0, 0.0, 1.0));
    add_triangle_color_normal(dat, &Triangle3{p1 : corners[2], p2 : corners[6], p3 : corners[7]}, &color, &Vector3::new(0.0, 0.0, 1.0));

    add_triangle_color_normal(dat, &Triangle3{p1 : corners[0], p2 : corners[2], p3 : corners[3]}, &color, &Vector3::new(0.0, -1.0, 0.0));
    add_triangle_color_normal(dat, &Triangle3{p1 : corners[2], p2 : corners[0], p3 : corners[1]}, &color, &Vector3::new(0.0, -1.0, 0.0));

    add_triangle_color_normal(dat, &Triangle3{p1 : corners[6], p2 : corners[4], p3 : corners[7]}, &color, &Vector3::new(0.0, 1.0, 0.0));
    add_triangle_color_normal(dat, &Triangle3{p1 : corners[6], p2 : corners[5], p3 : corners[4]}, &color, &Vector3::new(0.0, 1.0, 0.0));
}

pub fn add_sphere_color(dat : &mut RendererVertFragDef, sphere : &Sphere<f32>, n : usize, m : usize, color : Vector3<f32>){
    use std;
    let pi = std::f32::consts::PI;
    let dphi = 2.0 * pi / n as f32;
    let dpsi = pi / m as f32;

    for i in 0..n{
        let phi = i as f32 * dphi;
        let phi_next = (i + 1) as f32 * dphi;
        for j in 0..m{
            let psi = j as f32 * dpsi;
            let psi_next = (j + 1) as f32 * dpsi;

            let x0 = phi.cos() * psi.sin() * sphere.rad;
            let z0 = -phi.sin() * psi.sin() * sphere.rad;
            let y0 = -psi.cos() * sphere.rad;

            let x1 = phi_next.cos() * psi.sin() * sphere.rad;
            let z1 = -phi_next.sin() * psi.sin() * sphere.rad;
            let y1 = -psi.cos() * sphere.rad;

            let x2 = phi.cos() * psi_next.sin() * sphere.rad;
            let z2 = -phi.sin() * psi_next.sin() * sphere.rad;
            let y2 = -psi_next.cos() * sphere.rad;

            let x3 = phi_next.cos() * psi_next.sin() * sphere.rad;
            let z3 = -phi_next.sin() * psi_next.sin() * sphere.rad;
            let y3 = -psi_next.cos() * sphere.rad;

            let v0 = Vector3::new(x0, y0, z0);
            let v1 = Vector3::new(x1, y1, z1);
            let v2 = Vector3::new(x2, y2, z2);
            let v3 = Vector3::new(x3, y3, z3);

            let normal = (v1 - v0).cross(&(v2 - v0)).normalize();

             add_vector_to_pool(dat, sphere.center + v0);
             add_vector_to_pool(dat, color);
             add_vector_to_pool(dat, normal);

             add_vector_to_pool(dat, sphere.center + v1);
             add_vector_to_pool(dat, color);
             add_vector_to_pool(dat, normal);

             add_vector_to_pool(dat, sphere.center + v2);
             add_vector_to_pool(dat, color);
             add_vector_to_pool(dat, normal);

             add_vector_to_pool(dat, sphere.center + v3);
             add_vector_to_pool(dat, color);
             add_vector_to_pool(dat, normal);
            
        }
    }

    for i in 0..n*m{
        dat.index_pool.push(dat.vertex_count + 4*i as u32);
        dat.index_pool.push(dat.vertex_count + 4*i as u32 + 1);
        dat.index_pool.push(dat.vertex_count + 4*i as u32 + 2);

        dat.index_pool.push(dat.vertex_count + 4*i as u32 + 1);
        dat.index_pool.push(dat.vertex_count + 4*i as u32 + 3);
        dat.index_pool.push(dat.vertex_count + 4*i as u32 + 2);
    }

    dat.vertex_count += n as u32 * m as u32 * 4;

}

pub fn add_grid3_color(dat : &mut RendererVertFragDef, center : Vector3<f32>, tangent : Vector3<f32>, normal : Vector3<f32>, extent : f32, subdiv_num : u32, color : Vector3<f32>){
    let right = tangent.cross(&normal) * extent;
    let along = tangent * extent;
    add_vector_to_pool(dat, center - right - along);
    add_vector_to_pool(dat, color);
    add_vector_to_pool(dat, center + right - along);
    add_vector_to_pool(dat, color);
    add_vector_to_pool(dat, center + right + along);
    add_vector_to_pool(dat, color);
    add_vector_to_pool(dat, center - right + along);
    add_vector_to_pool(dat, color);

    let a = extent / subdiv_num as f32;
    //TODO inefficient loops(could be done in one)
    for i in 1 .. 2 * subdiv_num{
        add_vector_to_pool(dat, center - right * (extent - i as f32 * a) - along);
        add_vector_to_pool(dat, color);
    }

    for i in 1 .. 2 * subdiv_num{
        add_vector_to_pool(dat, center + right - along * (extent - i as f32 * a) );
        add_vector_to_pool(dat, color);
    }

    for i in 1 .. 2 * subdiv_num{
        add_vector_to_pool(dat, center + right * (extent - i as f32 * a) + along);
        add_vector_to_pool(dat, color);
    }

    for i in 1 .. 2 * subdiv_num{
        add_vector_to_pool(dat, center - right + along * (extent - i as f32 * a) );
        add_vector_to_pool(dat, color);
    }

    dat.index_pool.push(0 + dat.vertex_count);
    dat.index_pool.push(1 + dat.vertex_count);
    dat.index_pool.push(1 + dat.vertex_count);
    dat.index_pool.push(2 + dat.vertex_count);
    dat.index_pool.push(2 + dat.vertex_count);
    dat.index_pool.push(3 + dat.vertex_count);
    dat.index_pool.push(3 + dat.vertex_count);
    dat.index_pool.push(0 + dat.vertex_count);

    let off0 : u32 = 4;
    let off1 : u32 = subdiv_num * 2 - 1;

    for i in 0..off1{
        dat.index_pool.push(off0 + off1 + i + dat.vertex_count);
        dat.index_pool.push(off0 + 4*off1 - i - 1 + dat.vertex_count);
    }

    for i in 0..off1{
        dat.index_pool.push(off0 + i + dat.vertex_count);
        dat.index_pool.push(off0 + 3*off1 - i - 1 + dat.vertex_count);
    }

    dat.vertex_count += 4 + 4 * (2 * subdiv_num - 1) //TODO ???
}

