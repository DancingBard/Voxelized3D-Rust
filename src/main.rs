#![feature(box_syntax, box_patterns, clone_closures, copy_closures)]


extern crate generic_array;
extern crate nalgebra as na;
extern crate typenum;
extern crate alga;
extern crate libc;
extern crate ansi_term;
extern crate time;
extern crate num;
extern crate rand;
extern crate noise;

use na::*;
use na::core::Unit;

mod cms;
mod qef_bindings;
mod adaptive_dc;
mod graphics;
mod graphics_util;
mod renderer;
#[macro_use]
mod math;
mod voxel_renderer;
mod dc;
mod dcm;
mod matrix;
mod uniform_manifold_dc;
mod cubic;

use noise::{NoiseModule, Perlin};
use graphics::*;
use std::ptr;
use std::fs;
use std::fs::File;
use std::vec::*;
use std::collections::HashMap;
use graphics_util::*;
use std::io::Read;
use renderer::*;
use math::*;
use voxel_renderer::*;
use std::ops::*;
use rand::distributions::{Sample, Range};
use typenum::*;
use core::storage::*;
use generic_array::*;
use adaptive_dc::*;
use uniform_manifold_dc::*;

use time::precise_time_ns;

fn timed<T>(str_fn: &(Fn(u64) -> String), f : &mut (FnMut() -> T)) -> T{
    let t1 = precise_time_ns();
    let ret = f();
    let t2 = precise_time_ns();

    let dt = t2 - t1;

    println!("{}", str_fn(dt));

    ret
}

//F3 : FnMut(A) -> C
fn compose<'l, A, B, C, F1, F2>(f1 : & 'l Box<F1>, f2 : &'l Box<F2>) -> Box<Fn(A) -> C + 'l>
    where F1 : 'l + Fn(A) -> B,
          F2 : 'l + Fn(B) -> C,
          {
    Box::new(move |a : A| {(*f2)((*f1)(a))})
}



extern fn framebuf_sz_cb(win : *mut GlfwWindow, w : isize, h : isize){
    gl_viewport(0,0,w,h);
}

extern fn error_cb(n : isize, er : &str){
    println!("{}", er);
}

fn check_for_gl_errors(){
    let mut er: usize = gl_get_error();

    while er != GL_NO_ERROR{
        eprintln!("GL error: {}", er);
        er = gl_get_error();
    }
}

fn update_win_dim_info(info: &mut WindowInfo){
    let mut w: usize = 0;
    let mut h: usize = 0;

    glfw_get_window_size(info.handle, &mut w, &mut h);
    info.width = w;
    info.height = h;
}

fn process_input(win : *mut GlfwWindow, dt_ns : u64, camera : &mut Camera){
    if glfw_get_key(win, GLFW_KEY_ESCAPE) == GLFW_PRESS{
        glfw_set_window_should_close(win, true);
    }
    else if glfw_get_key(win, GLFW_KEY_TAB) == GLFW_PRESS{
        //debug

        let mut w : usize = 0;
        let mut h : usize = 0;

        glfw_get_window_size(win, &mut w, &mut h);

        println!("({}, {})", w, h);

        let mon = glfw_get_primary_monitor();
        let vid_mode = glfw_get_video_mode(mon);
        unsafe{
            println!("{:?}", *vid_mode)
        }
    }


    let dt_s : f32 = dt_ns as f32 / 1000000000.0;

    if glfw_get_key(win, GLFW_KEY_W) == GLFW_PRESS{
        camera.pos += camera.look * dt_s as f32;

    }

    if glfw_get_key(win, GLFW_KEY_S) == GLFW_PRESS{
        camera.pos -= camera.look * dt_s as f32;
    }

    if glfw_get_key(win, GLFW_KEY_A) == GLFW_PRESS{
        let right = camera.look.cross(&camera.up);

        camera.pos -= right * dt_s as f32;
    }

    if glfw_get_key(win, GLFW_KEY_D) == GLFW_PRESS{
        let right = camera.look.cross(&camera.up);

        camera.pos += right * dt_s as f32;
    }

    if glfw_get_key(win, GLFW_KEY_SPACE) == GLFW_PRESS{

        camera.pos += camera.up * dt_s as f32;
    }

    if glfw_get_key(win, GLFW_KEY_LEFT_SHIFT) == GLFW_PRESS{

        camera.pos -= camera.up * dt_s as f32;
    }

    if glfw_get_key(win, GLFW_KEY_LEFT) == GLFW_PRESS{

        let mat = na::Rotation3::from_axis_angle(&Unit::new_unchecked(camera.up), std::f32::consts::PI * dt_s / 2.0);
        camera.look = (mat * camera.look).normalize();
    }
    if glfw_get_key(win, GLFW_KEY_RIGHT) == GLFW_PRESS{

        let mat = na::Rotation3::from_axis_angle(&Unit::new_unchecked(camera.up), -std::f32::consts::PI * dt_s / 2.0);
        camera.look = (mat * camera.look).normalize();
    }
    if glfw_get_key(win, GLFW_KEY_KP_0) == GLFW_PRESS{

        let mat = na::Rotation3::from_axis_angle(&Unit::new_unchecked(camera.look), std::f32::consts::PI * dt_s / 2.0);
        camera.up = (mat * camera.up).normalize();
    }
    if glfw_get_key(win, GLFW_KEY_KP_DECIMAL) == GLFW_PRESS{

        let mat = na::Rotation3::from_axis_angle(&Unit::new_unchecked(camera.look), -std::f32::consts::PI * dt_s / 2.0);
        camera.up = (mat * camera.up).normalize();
    }
    if glfw_get_key(win, GLFW_KEY_RIGHT) == GLFW_PRESS{

        let mat = na::Rotation3::from_axis_angle(&Unit::new_unchecked(camera.up), -std::f32::consts::PI * dt_s / 2.0);
        camera.look = (mat * camera.look).normalize();
    }
    if glfw_get_key(win, GLFW_KEY_UP) == GLFW_PRESS{
        let right = camera.look.cross(&camera.up);
        let mat = na::Rotation3::from_axis_angle(&Unit::new_unchecked(right), std::f32::consts::PI * dt_s / 2.0);
        camera.look = (mat * camera.look).normalize();
        camera.up = (mat * camera.up).normalize();
    }
    if glfw_get_key(win, GLFW_KEY_DOWN) == GLFW_PRESS{
        let right = camera.look.cross(&camera.up);
        let mat = na::Rotation3::from_axis_angle(&Unit::new_unchecked(right), -std::f32::consts::PI * dt_s / 2.0);
        camera.look = (mat * camera.look).normalize();
        camera.up = (mat * camera.up).normalize();
    }

   /* println!("{}", camera.pos);
    println!("{}", camera.look.norm());
    println!("{}", camera.up.norm());*/
}


fn load_shaders_vf() -> HashMap<String, Program>{
    let dir : &str = "./assets/shaders/";
    let paths = fs::read_dir(dir).unwrap();
    let mut map : HashMap<String, Program> = HashMap::new();
    
    for entry in paths{
        let name : String = String::from(entry
            .unwrap()
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap());

        if !map.contains_key(&name){
            let mut file_vert = File::open(
                dir.to_string() + &name + ".vert").unwrap();
            let mut source_vert = String::new();
            file_vert.read_to_string(&mut source_vert).unwrap();

            let mut file_frag = File::open(
                dir.to_string() + &name + ".frag").unwrap();
            let mut source_frag = String::new();
            file_frag.read_to_string(&mut source_frag).unwrap();

            let prog = create_program_vf(
                &source_vert,
                &source_frag);
            
            
            map.insert(name, Program{id: prog});
        }
    }
    
    map
}

unsafe fn compose2<'a, A,B,C, F, G>(f : *const F, g : *const G) -> Box<Fn(A) -> C + 'a> where
    F :  Fn(A) -> B + 'a,
    G :  Fn(B) -> C + 'a{
    unsafe{
        box move |a| (*g)( (*f)(a) )
    }
}


fn main(){
   
    let id = |x : usize| x;
    unsafe{
       let f1 = compose2(&id, &id);
       f1(1);
       id(1);

       let rec = mk_rectangle2(Vector2::new(0.0, 0.0), Vector2::new(1.0, 2.0));
       println!("{}", rec(Vector2::new(1.0,0.0)));
    }


    
    //matrix::test_matrices();
    run_voxelized();
}




fn run_voxelized() {
    let def_width: usize = 800;
    let def_height: usize = 600;

    //TODO check if it works
    glfw_set_error_callback(error_cb);
    glfw_init();
    glfw_window_hint(GLFW_CONTEXT_VERSION_MAJOR, 3);
    glfw_window_hint(GLFW_CONTEXT_VERSION_MINOR, 3);
    glfw_window_hint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

    let win = glfw_create_window(def_width, def_height, "Voxelized2D");

    if win == ptr::null_mut(){
        glfw_terminate();
        panic!("Failed to create GLFW window !");
    }

    glfw_make_context_current(win);
    glad_load_gl_loader();

    println!("Using GL version: {}", gl_get_string(GL_VERSION));
    
    glfw_set_framebuffer_size_callback(win, framebuf_sz_cb);

    glfw_set_input_mode(win, GLFW_STICKY_KEYS, 1);

    let shaders = load_shaders_vf();

    let mut camera = Camera{pos : Vector3::new(0.0,0.0,0.0), look : Vector3::new(0.0,0.0,-1.0), up : Vector3::new(0.0, 1.0, 0.0)};


    let mut voxel_renderer = VoxelRenderer::new(&shaders);
    let mut win_info = WindowInfo{width: def_width, height: def_height, handle: win}; //will be updated each frame




    let mut renderer_tr_light = RendererVertFragDef::make(
        VERTEX_SIZE_COLOR_NORMAL,
        set_attrib_ptrs_color_normal,
        GL_TRIANGLES,
        String::from("lighting"));

    let mut renderer_lines = RendererVertFragDef::make(
        VERTEX_SIZE_COLOR,
        set_attrib_ptrs_color,
        GL_LINES,
        String::from("color")
    );

    let zero = Vector3::new(0.0, 0.0, 0.0);
    let offset = Vector3::new(0.1, 0.1, 0.1);
    let red = Vector3::new(1.0, 0.0, 0.0);
    let green = Vector3::new(0.0, 1.0, 0.0);
    let blue = Vector3::new(0.0, 0.0, 1.0);
    let white = Vector3::new(1.0, 1.0, 1.0);
    


    unsafe{
        cubic::test_cubic_octree(&mut renderer_tr_light);
    }

    add_grid3_color(&mut renderer_lines, zero, Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 1.0, 0.0), 1.0, 8, white);

    add_line3_color(&mut renderer_lines, Line3{start : zero, end : zero + red}, red);
    add_line3_color(&mut renderer_lines, Line3{start : zero, end : zero + green}, green);
    add_line3_color(&mut renderer_lines, Line3{start : zero, end : zero + blue}, blue);

    //add_triangle_color_normal(&mut renderer_tr_light, &Triangle3{p1 : Vector3::new(-0.3, 0.0, -1.0), p2 : Vector3::new(0.3, 0.0, -1.0), p3 : Vector3::new(0.0, 0.5, -1.0)}, &red, &Vector3::new(0.0, 0.0, 1.0));

    //add_square3_bounds_color(&mut renderer_lines, Square3{center : Vector3::new(-0.5, 0.5, -0.5), extent : 0.125 / 2.0}, red + green);




    //====================================
    let BLOCK_SIZE : f32 = 0.125;//2.0;
    let CHUNK_SIZE : usize = 128;//*2;

    let mut grid = dcm::VoxelMaterialGrid3::new(BLOCK_SIZE, CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE);




   

    let sphere1_ = Sphere{center : Vector3::new(4.1 as f32,4.2, 4.3), rad : 2.2};
    let sphere2_ = Sphere{center : Vector3::new(3.0 ,3.0, 3.0), rad : 1.0};
    let sphere1 = dcm::mk_sphere_mat(sphere1_.clone(), 1);
    let sphere11 = dcm::mk_sphere_mat(sphere1_.clone(), 1);
    let sphere2 = dcm::mk_sphere_mat(sphere2_.clone(), 2);
    //let aabb = mk_aabb(Vector3::new(4.0, 5.0, 4.0), Vector3::new(1.0, 1.0, 1.0));

    /* let perlin = Perlin::new();
    let val = perlin.get([42.4, 37.7, 2.8]);
    
    let sphere_disp = mk_sphere_displacement(sphere1_.clone(), box move |x| {
         perlin.get([x.x/2.0, x.y/2.0,x.z/2.0]) + 1.0
         
    }); */

    let den1 = dcm::union3_mat(sphere11, sphere2);
    let den = dcm::intersection3_mat_a(den1, sphere1);
    //let den = union3(den1, aabb);

    //dc::test_sample_normal();


    //ADAPTIVE---------
    // let sp_num = mk_sphere(Sphere{center : Vector3::new(-4.0, -4.0, -4.0), rad : 1.0});
    // let tree = timed(&|dt| format!("make tree took {} ms", dt / 1000000), &mut ||{
    //    make_tree(&sp_num, Vector3::new(-5.0, -5.0, -5.0), BLOCK_SIZE, CHUNK_SIZE, &mut renderer_lines)
    // });

    //-----------------

    //UNIFORM MANIFOLD DC
    let sp_num1 = mk_sphere(Sphere{center : Vector3::new(2.0, 2.0, -1.0), rad : 1.0});
    let sp_num2 = mk_sphere(Sphere{center : Vector3::new(2.0, 2.0, 1.001), rad : 1.0});
    let rec1 = mk_aabb(Vector3::new(2.0,2.0,0.0), Vector3::new(0.2,0.2,0.2));
    let den1 = union3(sp_num1, sp_num2);
    let den = union3(rec1, den1);
    let torusz = mk_torus_z(2.0, 0.8,Vector3::new(0.0,0.0,-4.0));
    let torusy = mk_torus_y(1.6, 0.67,Vector3::new(2.0,0.0,-4.0));
    let perlin = Perlin::new();
    let noise = noise_f32(perlin, Square3{center : Vector3::new(1.0,-1.0,1.0), extent : 3.5} );//perlin.get([p.x,p.y,p.z])  ;
    let two_torus = union3(torusz, torusy);
    let den2 = difference3(two_torus, mk_aabb(Vector3::new(0.0, 3.0, -4.0), Vector3::new(1.5,1.5,1.5)));
    let den3 = union3(den2, mk_sphere(Sphere{center : Vector3::new(0.0, 2.0, -4.0), rad : 1.0}));
    let den4 = union3(den3, mk_obb(Vector3::new(1.0, 1.0, 0.0), Vector3::new(1.0, -1.0, 0.0).normalize(), Vector3::new(1.0, 1.0, 0.5).normalize(), Vector3::new(1.0, 0.5, 0.2)));
    //let den4 = union3(den3, mk_half_space_pos(Plane{point : Vector3::new(0.0, 2.0, -4.0), normal : Vector3::new(1.0, 1.0, 0.0).normalize()}));
    //let den = f;
    //TODO implement DenFn differently, like noise library
    //construct_grid(&den4, Vector3::new(-3.0, -3.0, -8.0), BLOCK_SIZE, CHUNK_SIZE, 8, &mut renderer_tr_light, &mut renderer_lines);

    let test_sphere = Sphere{center : Vector3::new(2.7, 1.0, 0.0), rad : 2.4};
    let test_sphere2 = Sphere{center : Vector3::new(2.7, 3.0, 0.0), rad : 2.4};
    let test_sphere3 = Sphere{center : Vector3::new(2.7, 1.0, 2.7), rad : 1.4};
    let ts1 = mk_sphere(test_sphere);
    let ts2 = mk_sphere(test_sphere2);
    let ts22 = mk_sphere(test_sphere3);
    let ts3 = difference3(ts1, ts2);
    let ts4 = difference3(ts3, ts22);
    //add_sphere_color(&mut renderer_tr_light, &test_sphere, 100, 100, Vector3::new(1.0, 1.0, 1.0));
    //construct_grid(&ts4, Vector3::new(-0.5, -2.5, -2.5), 1.0/8.0, 2*8*8, 32, &mut renderer_tr_light, &mut renderer_lines);
    ///------------------

    // let contour_data = timed(&|dt| format!("op took {} ms", dt / 1000000), &mut ||{
    //     dcm::fill_in_grid(&mut grid, &den, Vector3::new(0.0, 0.0, 0.0));
    //     dcm::make_contour(&grid, &den, 16, &mut renderer_lines) //accurary depends on grid resolution
    // });


    shaders.get("lighting").unwrap().enable();
    shaders.get("lighting").unwrap().set_vec3f("pointLight.pos" ,Vector3::new(0.0, 8.0,0.0));
    shaders.get("lighting").unwrap().set_vec3f("pointLight.color" ,(red + green + blue) * 15.0);

    // println!("generated {} triangles", contour_data.triangles.len());

    // for i in 0..contour_data.triangles.len(){
    //     //add_triangle_color_normal(&mut renderer_tr_light, &contour_data.triangles[i], &contour_data.triangle_colors[i / 2], &contour_data.triangle_normals[i / 2]);
    // }
    //===================================


    fn shader_data(shader: &Program, win: &WindowInfo, camera : &Camera) -> bool{
        let aspect = win.width as f32 / win.height as f32;
       /* let height = 16.0;
        let width = height;*/
        let id_mat = [
            1.0,0.0,0.0,0.0,
            0.0,1.0,0.0,0.0,
            0.0,0.0,1.0,0.0,
            0.0,0.0,0.0,1.0];




        let persp = perspective(90.0, aspect, 0.1, 16.0);
        let view = view_dir(camera.pos, camera.look, camera.up);

        println!("{}", persp);


        shader.set_float4x4("P", false, persp.as_slice());
        shader.set_float4x4("V", false, view.as_slice());

        true

    };

    fn shader_data_lines(shader: &Program, win: &WindowInfo, camera : &Camera) -> bool{
        let aspect = win.width as f32 / win.height as f32;
       /* let height = 16.0;
        let width = height;*/
        let id_mat = [
            1.0,0.0,0.0,0.0,
            0.0,1.0,0.0,0.0,
            0.0,0.0,1.0,0.0,
            0.0,0.0,0.0,1.0];




        let persp = perspective(90.0, aspect, 0.1, 16.0);
        let view = view_dir(camera.pos, camera.look, camera.up);


        shader.set_float4x4("P", false, persp.as_slice());
        shader.set_float4x4("V", false, view.as_slice());
   
        glfw_get_key(win.handle, GLFW_KEY_TAB) != GLFW_PRESS
    };

    let provider = RenderDataProvider{pre_render_state: None, post_render_state: None, shader_data: Some(Box::new(shader_data))};
    let provider_lines = RenderDataProvider{pre_render_state: None, post_render_state: None, shader_data: Some(Box::new(shader_data_lines))};


    let mut render_info = RenderInfo{renderer: Box::new(renderer_tr_light), provider};//moved
    let mut render_info_lines = RenderInfo{renderer: Box::new(renderer_lines), provider: provider_lines}; //moved


    let id_trs = voxel_renderer.push(RenderLifetime::Manual, RenderTransform::None, render_info).unwrap();
    let id_lns = voxel_renderer.push(RenderLifetime::Manual, RenderTransform::None, render_info_lines).unwrap();


    voxel_renderer.manual_mut(&id_trs).construct();
    voxel_renderer.manual_mut(&id_lns).construct();

    let mut last_frame_time = precise_time_ns();
    let mut cur_frame_time = last_frame_time;


   gl_enable(GL_DEPTH_TEST);
    while !glfw_window_should_close(win){

        gl_clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT); 

        last_frame_time = cur_frame_time;
        cur_frame_time = precise_time_ns();

        let dt_ns = cur_frame_time - last_frame_time;


        update_win_dim_info(&mut win_info);
        process_input(win, dt_ns, &mut camera);

        gl_clear_color(0.2, 0.3, 0.3, 1.0);

        voxel_renderer.draw(&win_info, &camera);


        glfw_swap_buffers(win);
        glfw_poll_events();

        check_for_gl_errors();
    }

    voxel_renderer.manual_mut(&id_trs).deconstruct();
    voxel_renderer.manual_mut(&id_trs).reset();

    voxel_renderer.manual_mut(&id_lns).deconstruct();
    voxel_renderer.manual_mut(&id_lns).reset();

    glfw_terminate();
}


/* pub struct VoxelGrid2<T : Real + Copy>{
    pub a : T,
    pub size_x : usize,
    pub size_y : usize,
    pub grid : Vec<T>,
}

impl<T : Real + SupersetOf<f32>> VoxelGrid2<T>{

    pub fn vertices_x(&self) -> usize {self.size_x + 1}
    pub fn vertices_y(&self) -> usize {self.size_y + 1}

    pub fn new(a : T, size_x : usize, size_y : usize) -> VoxelGrid2<T>{
        let grid = vec![convert(0.0);(size_x + 1) * (size_y + 1)];

        VoxelGrid2{a,size_x, size_y, grid}
    }

    pub fn get(&self, x : usize, y : usize) -> T{
        self.grid[y * self.vertices_x() + x]
    }

    pub fn set(&mut self, x : usize, y : usize, value : T){
        let vx = self.vertices_x();
        self.grid[y * vx + x] = value;
    }

    pub fn get_point(&self, x : usize, y : usize) -> Vector2<T>{
        Vector2::new(self.a * convert::<f32, T>(x as f32), self.a * convert::<f32, T>(y as f32))
    }

    pub fn square2(&self, x : usize, y : usize) -> Square2<T>{
        Square2{center : Vector2::new(convert::<f32,T>(x as f32 + 0.5) * self.a, convert::<f32,T>(y as f32 + 0.5) * self.a), extent: self.a / convert(2.0)}
    }
} */


/*fn calc_qef(point : &Vector2<f32>, lines : &Vec<Line2<f32>>) -> f32{
    let mut qef : f32 = 0.0;
    for line in lines{
        let dist = distance_point2_line2(point, line);
        qef += dist * dist;
    }

    qef
}

fn const_sign(a : f32, b : f32) -> bool {
    if a > 0.0 { b > 0.0} else {b <= 0.0}
}

fn sample_qef_brute(square : Square2<f32>, n : usize, lines : &Vec<Line2<f32>>) -> Vector2<f32> {
    let ext = Vector2::new(square.extent, square.extent);
    let min = square.center - ext;

    let mut best_qef = 100000000000.0; //TODO placeholder
    let mut best_point = min;

    for i in 0..n{
        for j in 0..n{
            let point = min + Vector2::new(ext.x * (2.0 * (i as f32) + 1.0) / (n as f32), ext.y * (2.0 * (j as f32) + 1.0) / (n as f32));
            let qef = calc_qef(&point, &lines);

            if qef < best_qef{
                best_qef = qef;
                best_point = point;
            }
        }
    }

    best_point
}


fn sample_intersection_brute(line : Line2<f32>, n : usize, f : &DenFn2<f32>) -> Vector2<f32>{
    let ext = line.end - line.start;

    let mut best_abs = 1000000000.0; //TODO placeholder
    let mut best_point : Option<Vector2<f32>> = None;

    for i in 0..n {
        let point = line.start + ext * (i as f32 / n as f32);
        let den = f(point);
        let abs = den.abs();

        if abs < best_abs {
            best_abs = abs;
            best_point = Some(point);
        }
    }

    best_point.unwrap()
}

fn sample_tangent(square : Square2<f32>, n : usize, f : &DenFn2<f32>) -> Vector2<f32>{
    let ext = Vector2::new(square.extent, square.extent);
    let min = square.center - ext;

    let den_at_center = f(square.center);

    let mut closest = den_at_center + 100000000.0; //TODO placeholder\
    let mut closest_point = square.center;

    for i in 0..n{
        for j in 0..n{
            let point = min + Vector2::new(ext.x * (2.0 * i as f32) / n as f32,
                ext.y * (2.0 * j as f32) / n as f32);
            let den = f(point);
            let attempt = (den - den_at_center).abs();
            if attempt < closest && (point - square.center).norm() != 0.0{
                closest = attempt;
                closest_point = point;
            }
        }
    }

    closest_point - square.center
}

fn ext_for_normal(block_size : f32) -> f32 {block_size / 100.0} //TODO why so ?


fn make_lines(vg : &VoxelGrid2<f32>, features : &Vec<Option<Vector2<f32>>>) -> Vec<Line2<f32>>{
    let mut ret = Vec::<Line2<f32>>::new();

    for y in 0..vg.size_y - 1{
        for x in 0..vg.size_x - 1{
            let feature = features[y * vg.size_x + x];
            if feature.is_some(){
                let p1 = vg.get(x + 1, y);
                let p2 = vg.get(x, y + 1);
                let p3 = vg.get(x + 1, y + 1);

                let mut vert1 : Option<Vector2<f32>> = None;
                let mut vert2 : Option<Vector2<f32>> = None;

                if !const_sign(p1,p3){
                    vert1 = features[y * vg.size_x + (x + 1)];
                }
                if !const_sign(p3,p2){
                    vert2 = features[(y+1) * vg.size_x + x];
                }

                if vert1.is_some(){
                    ret.push(Line2{start : feature.unwrap(), end : vert1.unwrap()});
                }
            }
        }
    }

    ret
}

fn make_triangles(vg : &VoxelGrid2<f32>, features : &Vec<Option<Vector2<f32>>>, intersections : &Vec<Option<Vec<Vector2<f32>>>>,
    extra : &Vec<Option<Vec<Vector2<f32>>>>) -> Vec<Triangle2<f32>>{
    let mut ret = Vec::<Triangle2<f32>>::new();

    for y in 0..vg.size_y{
        for x in 0.. vg.size_x{
            let t = y * vg.size_x + x;
            let cur_intersections = &intersections[t];
            let cur_extras = &extra[t];

            let p0 = vg.get(x, y);
            let p1 = vg.get(x + 1, y);
            let p2 = vg.get(x, y + 1);
            let p3 = vg.get(x + 1, y + 1);

            let v0 = vg.get_point(x,y);
            let v1 = vg.get_point(x + 1, y);
            let v2 = vg.get_point(x, y + 1);
            let v3 = vg.get_point(x + 1, y + 1);

            let mut sit = 0;

            if !const_sign(p0, p1){sit |= 1;}
            if !const_sign(p1, p3){sit |= 2;}
            if !const_sign(p3, p2){sit |= 4;}
            if !const_sign(p2, p0){sit |= 8;}

            if sit == 0{ //fully inside or fully outside
                let negative = p0 < 0.0;

                if negative{ //render if it is inside
                    let tr1 = Triangle2{p1: v0, p2 : v1, p3 : v3};
                    let tr2 = Triangle2{p1: v0, p2 : v3, p3 : v2};

                    ret.push(tr1);
                    ret.push(tr2);
                }

            }else{ //contains surface
                if cur_intersections.is_some() && features[t].is_some(){
                    let len = cur_intersections.as_ref().unwrap().len();
                    for i in 0..len{
                        ret.push(Triangle2{p1 : features[t].as_ref().unwrap().clone(), p2 : cur_intersections.as_ref().unwrap()[i].clone(), p3 : cur_extras.as_ref().unwrap()[i].clone()});
                    }
                }
            }
        }
    }

    ret
}


fn make_vertex(vg : &VoxelGrid2<f32>, tr : &mut Vec<Triangle2<f32>>, x : usize, y : usize,
    f : &DenFn2<f32>, accuracy : usize, features : &mut Vec<Option<Vector2<f32>>>, out_intersections : &mut Vec<Vector2<f32>>, out_extra : &mut Vec<Vector2<f32>>) -> Option<Vector2<f32>>{
    let epsilon = vg.a / accuracy as f32;

    let p0 = vg.get(x, y);
    let p1 = vg.get(x + 1, y);
    let p2 = vg.get(x, y + 1);
    let p3 = vg.get(x + 1, y + 1);

    let v0 = vg.get_point(x,y);
    let v1 = vg.get_point(x + 1, y);
    let v2 = vg.get_point(x, y + 1);
    let v3 = vg.get_point(x + 1, y + 1);

    let mut sit = 0;

    if !const_sign(p0, p1){sit |= 1;}
    if !const_sign(p1, p3){sit |= 2;}
    if !const_sign(p3, p2){sit |= 4;}
    if !const_sign(p2, p0){sit |= 8;}

    let ext_for_normal = ext_for_normal(vg.a);

    if sit > 0{
        let mut tangents = Vec::<Line2<f32>>::new();

        let mut vert1 : Option<Vector2<f32>> = None;
        let mut vert2 : Option<Vector2<f32>> = None;

        {
            let mut worker = |and : usize, v_a : Vector2<f32>, v_b : Vector2<f32>, p_a : f32, p_b : f32|{
                if (sit & and) > 0{
                    let ip = sample_intersection_brute(Line2{start : v_a, end : v_b}, accuracy, f);
                    let full = if p_a <= 0.0 {v_a} else {v_b};
                    let dir = sample_tangent(Square2{center : ip, extent : ext_for_normal}, accuracy, f);
                    let line = Line2{start : ip - dir * (1.0 / ext_for_normal), end : ip + dir * (1.0 / ext_for_normal)};
                    tangents.push(line);

                    out_intersections.push(ip);
                    out_extra.push(full);

                }else{
                    let negative = p_a < 0.0;
                    if negative{
                        out_intersections.push(v_a);
                        out_extra.push(v_b);
                    }
                }
            };

            worker(1, v0, v1, p0, p1);
            worker(2, v1, v3, p1, p3);
            worker(4, v3, v2, p3, p2);
            worker(8, v2, v0, p2, p0);
        }

        let interpolated_vertex = sample_qef_brute(vg.square2(x,y), accuracy, &tangents);

        for i in 0..out_intersections.len(){
            tr.push(Triangle2{p1 : interpolated_vertex, p2 : out_intersections[i], p3 : out_extra[i]});
        }

        features[y * vg.size_x + x] = Some(interpolated_vertex);

        Some(interpolated_vertex)
    }else{
        None
    }
}

struct ContourData{
    pub lines : Vec<Line2<f32>>,
    pub triangles : Vec<Triangle2<f32>>,
    pub features : Vec<Option<Vector2<f32>>>,
    pub intersections : Vec<Option<Vec<Vector2<f32>>>>,
    pub extras : Vec<Option<Vec<Vector2<f32>>>>,
}

fn make_contour(vg : &VoxelGrid2<f32>, f : &DenFn2<f32>, accuracy : usize) -> ContourData{
    let mut res1 = Vec::<Line2<f32>>::new();
    let mut res2 = Vec::<Triangle2<f32>>::new();

    let mut features : Vec<Option<Vector2<f32>>> = vec![None;vg.size_x * vg.size_y];
    let mut intersections : Vec<Option<Vec<Vector2<f32>>>> = vec![None;vg.size_x * vg.size_y];
    let mut extras : Vec<Option<Vec<Vector2<f32>>>> = vec![None;vg.size_x * vg.size_y];

    {
        let mut cached_make = |x: usize, y: usize, res2: &mut Vec<Triangle2<f32>>| -> Option<Vector2<f32>>{
            let t = y * vg.size_x + x;
            let possible = features[t];
            if possible.is_none() {
                intersections[t] = Some(Vec::with_capacity(4));//TODO extra mem usage
                extras[t] = Some(Vec::with_capacity(4));

                let ret = make_vertex(vg, res2, x, y, f, accuracy, &mut features, &mut intersections[t].as_mut().unwrap(), &mut extras[t].as_mut().unwrap());
                if ret.is_none() {
                    intersections[t] = None;
                    extras[t] = None;
                }

                ret
            } else {
                possible
            }
        };

        for y in 0..vg.size_y {
            for x in 0..vg.size_x {
                let p0 = vg.get(x, y);
                let p1 = vg.get(x + 1, y);
                let p2 = vg.get(x, y + 1);
                let p3 = vg.get(x + 1, y + 1);

                let v0 = vg.get_point(x, y);
                let v1 = vg.get_point(x + 1, y);
                let v2 = vg.get_point(x, y + 1);
                let v3 = vg.get_point(x + 1, y + 1);

                let mut sit = 0;

                if !const_sign(p0, p1) { sit |= 1; }
                if !const_sign(p1, p3) { sit |= 2; }
                if !const_sign(p3, p2) { sit |= 4; }
                if !const_sign(p2, p0) { sit |= 8; }

                if sit > 0 {
                    let interpolated_vertex = cached_make(x, y, &mut res2).unwrap(); //it is 'some' here

                    let mut vert1: Option<Vector2<f32>> = None;
                    let mut vert2: Option<Vector2<f32>> = None;

                    if (sit & 2) > 0 {
                        if x + 1 < vg.size_x {
                            vert1 = cached_make(x + 1, y, &mut res2);
                        }
                    }
                    if (sit & 4) > 0 {
                        if y + 1 < vg.size_y {
                            vert2 = cached_make(x, y + 1, &mut res2);
                        }
                    }

                    if vert1.is_some() {
                        res1.push(Line2 { start: interpolated_vertex, end: vert1.unwrap() });
                    }
                    if vert2.is_some() {
                        res1.push(Line2 { start: interpolated_vertex, end: vert2.unwrap() });
                    }
                } else {
                    let negative = p0 < 0.0;

                    if negative {
                        let tr1 = Triangle2 { p1: v0, p2: v1, p3: v3 };
                        let tr2 = Triangle2 { p1: v0, p2: v3, p3: v2 };

                        res2.push(tr1);
                        res2.push(tr2);
                    }
                }
            }
        }
    }

    ContourData{lines : res1, triangles : res2, features, intersections, extras}

}

fn fill_in_grid(vg : &mut VoxelGrid2<f32>, f : &DenFn2<f32>, point : Vector2<f32>){
    for y in 0..vg.vertices_y(){
        for x in 0..vg.vertices_x(){
            let vx = vg.vertices_x();
            vg.grid[y * vx + x] = f(point + Vector2::new(vg.a * (x as f32), vg.a * (y as f32)));
        }
    }
}*/

