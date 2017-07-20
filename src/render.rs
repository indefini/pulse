use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use libc::c_int;
use dormin::render::{RenderPass,TransformCamera,TransformMeshRender, MatrixMeshRender, CameraIdMat};
use dormin::{resource, camera, vec, mesh, mesh_render, geometry, shader, render, fbo};
use dormin::resource::ResTT;

use dormin::material;

pub struct Render
{
    passes : HashMap<String, Box<RenderPass>>, //TODO check

    resource : Rc<resource::ResourceGroup>,

    //camera_ortho : camera::Camera,
    camera_ortho : TransformCamera,

    //fbo_all : Arc<RwLock<fbo::Fbo>>,
    //fbo_selected : Arc<RwLock<fbo::Fbo>>,
    fbo_all : usize,
    fbo_selected : usize,

    quad_outline : TransformMeshRender,
    quad_all : TransformMeshRender,

    //shoud be view objects?
    grid : TransformMeshRender,
    camera_repere : TransformMeshRender,

    line : TransformMeshRender,
}

impl Render {

    //TODO remove dragger and put "view_objects"
    pub fn new(resource : Rc<resource::ResourceGroup> ) -> Render
    {
        let fbo_all = resource.fbo_manager.borrow_mut().request_use_no_proc_new("fbo_all");
        let fbo_selected = resource.fbo_manager.borrow_mut().request_use_no_proc_new("fbo_selected");

        let camera_ortho =
        {
            let mut cam = TransformCamera::new();
            cam.data.projection = camera::Projection::Orthographic;
            cam.pan(&vec::Vec3::new(0f64,0f64,50f64));
            cam
        };

        let quad_outline = {
            let mut m = mesh::Mesh::new();
            m.add_quad(1f32, 1f32);

            let outline_mat = create_outline_material(
                &mut *resource.material_manager.borrow_mut(),
                &mut *resource.shader_manager.borrow_mut(),
                );

            let mere = mesh_render::MeshRender::new_with_mesh_and_mat_res(ResTT::new_with_instance("outline_quad", m), outline_mat);
             TransformMeshRender::with_mesh(mere)
        };

        let quad_all = {
            let mut m = mesh::Mesh::new();
            m.add_quad(1f32, 1f32);

            //let all_mat = resource.material_manager.borrow_mut().request_use_no_proc_tt_instance("material/fbo_all.mat");
            let all_mat = create_all_material(
                &mut*resource.material_manager.borrow_mut(),
                &mut*resource.shader_manager.borrow_mut());

            let mere = mesh_render::MeshRender::new_with_mesh_and_mat_res(ResTT::new_with_instance("quad_all", m), all_mat);
             TransformMeshRender::with_mesh(mere)
        };

        let line_mat = create_line_material(
            &mut*resource.material_manager.borrow_mut(),
            &mut*resource.shader_manager.borrow_mut());

        let grid = {
            let mut m = mesh::Mesh::new();
            create_grid(&mut m, 100i32, 1i32);

            let mere = mesh_render::MeshRender::new_with_mesh_and_mat_res(
                ResTT::new_with_instance("none", m),
                line_mat.clone());
             TransformMeshRender::with_mesh(mere)
        };

        let camera_repere = {
            let mut m = mesh::Mesh::new();
            create_repere(&mut m, 40f64 );

            let mere = mesh_render::MeshRender::new_with_mesh_and_mat_res(
                ResTT::new_with_instance("none", m),
                line_mat.clone());
            TransformMeshRender::with_mesh(mere)
        };

        let line =
        {
            let m = mesh::Mesh::new();
            let mere = mesh_render::MeshRender::new_with_mesh_and_mat_res(
                ResTT::new_with_instance("none", m),
                line_mat);
            TransformMeshRender::with_mesh(mere)
        };

        Render { 
            passes : HashMap::new(),
            camera_ortho : camera_ortho,
            fbo_all : fbo_all,
            fbo_selected : fbo_selected,
            quad_outline : quad_outline,
            quad_all : quad_all, 


            grid : grid,
            camera_repere : camera_repere,

            line : line, 
            resource : resource.clone()
        }
    }

    pub fn init(&mut self)
    {
        //self.fbo_all.write().unwrap().cgl_create();
        let mut fbo_mgr = self.resource.fbo_manager.borrow_mut();
        {
            let fbo_all = fbo_mgr.get_mut_or_panic(self.fbo_all);
            //fbo_all.write().unwrap().cgl_create();
            fbo_all.cgl_create();
        }

        let fbo_sel = fbo_mgr.get_mut_or_panic(self.fbo_selected);
        fbo_sel.cgl_create();
    }

    pub fn resize(&mut self, w : c_int, h : c_int)
    {
        {
            self.quad_outline.transform.scale = 
                vec::Vec3::new(w as f64, h as f64, 1f64);

            self.quad_all.transform.scale = 
                vec::Vec3::new(w as f64, h as f64, 1f64);

            self.camera_ortho.set_resolution(w, h);

            //self.fbo_all.write().unwrap().cgl_resize(w, h);
            //let fbo_all = &self.resource.fbo_manager.borrow().get_from_state(self.fbo_all);
            //fbo_all.write().unwrap().cgl_resize(w, h);
            let mut fbo_mgr = self.resource.fbo_manager.borrow_mut();
            {
                let fbo_all = fbo_mgr.get_mut_or_panic(self.fbo_all);
                fbo_all.cgl_resize(w, h);
                //    self.fbo_selected.write().unwrap().cgl_resize(w, h);
            }

            let fbo_sel = fbo_mgr.get_mut_or_panic(self.fbo_selected);
            fbo_sel.cgl_resize(w,h);
        }

        self.resolution_set(w,h);
    }


    fn resolution_set(&mut self, w : c_int, h : c_int)
    {
        self.quad_outline.set_uniform_data(
            "resolution",
            shader::UniformData::Vec2(vec::Vec2::new(w as f64, h as f64)));

        /*
        self.quad_all.clone().read().unwrap().set_uniform_data(
            "resolution",
            shader::UniformData::Vec2(vec::Vec2::new(w as f64, h as f64)));
            */
    }

    fn clean_passes(&mut self)
    {
        for (_,p) in self.passes.iter_mut()
        {
            //p.objects.clear();
            p.passes.clear();
        }

        {
            let mesh = &mut self.line.mr.mesh.get_instance().unwrap();//.write().unwrap();
            mesh.clear_lines();
        }
    }


    //TODO draw armature
    /*
    fn add_objects_to_passes(
        &mut self,
        camera : &CameraIdMat,
        objects : &[Arc<RwLock<object::Object>>]
        )
    {
        for o in objects.iter() {
            prepare_passes_object(
                o.clone(),
                &mut self.passes,
                &mut self.resource.material_manager.borrow_mut(),
                &mut self.resource.shader_manager.borrow_mut(),
                camera);
            
            let ob =  &*o.read().unwrap();
            match ob.get_component::<armature_animation::ArmatureAnimation>() {
                Some(ref aa) => {
                    let armature = &aa.arm_instance;
                    let color = vec::Vec4::new(1f64,1f64,1f64,1.0f64);

                    let line : &mut object::Object = &mut *self.line.write().unwrap();
                    if let Some(ref mut mr) = line.mesh_render
                    {
                        let mesh = &mut mr.mesh.get_instance().unwrap();

                        let arm_pos = ob.position + ob.orientation.rotate_vec3(&(armature.position*ob.scale));
                        let cur_rot = ob.orientation.as_quat() * armature.rotation;

                        for i in 0..armature.get_bones().len() {
                            let b = armature.get_bone(i);
                            let current_bone_position = armature.position_relative[i];
                            let current_bone_rotation = cur_rot*armature.rotation_relative[i];
                            let p1 = arm_pos + cur_rot.rotate_vec3(&(current_bone_position*ob.scale));
                            let bone_length = (b.tail - b.head)*ob.scale;
                            let diff = current_bone_rotation.rotate_vec3(&bone_length);
                            let p2 = p1 + diff;
                            let s = geometry::Segment::new(p1,p2);
                            mesh.add_line(s, color);
                        }
                    }
                }
                None => {}// println!("{} nooooooo", ob.name)}
            };
        }
    }
    */

    fn prepare_passes_objects_ortho(&mut self, mmr : &[MatrixMeshRender])
    {
        for (_,p) in self.passes.iter_mut()
        {
            //p.objects.clear();
            p.passes.clear();
        }

        let cam = CameraIdMat::from_transform_camera(&self.camera_ortho);
        self.add_mmr(&cam, &mmr);
    }

    fn prepare_passes_objects_per_mmr(
        &mut self,
        camera : &CameraIdMat,
        mmr : &[MatrixMeshRender])
    {
        let load = Arc::new(Mutex::new(0));
        for (_,p) in self.passes.iter_mut()
        {
            p.passes.clear();
        }

        self.add_mmr(camera, mmr);
    }

    fn add_mmr(
        &mut self,
        camera : &CameraIdMat,
        mmr : &[MatrixMeshRender])
    {
        let load = Arc::new(Mutex::new(0));

        for m in mmr {
            let pass = render::get_pass_from_mesh_render(
                &m.mr,
                &mut self.passes,
                &mut self.resource.material_manager.borrow_mut(),
                &mut self.resource.shader_manager.borrow_mut(),
                camera,
                load.clone()
                );

            if let Some(cam_pass) = pass {
                cam_pass.add_mmr(m.clone());
            }
        }
    }

    pub fn draw(
        &mut self,
        camera : &CameraIdMat,
        objects : &[MatrixMeshRender],
        cameras : &[MatrixMeshRender],
        selected : &[MatrixMeshRender],
        draggers : &[MatrixMeshRender],
        on_finish : &Fn(bool),
        load : Arc<Mutex<usize>>
        ) -> usize
    {
        let mut not_loaded = 0;
        self.prepare_passes_objects_per_mmr(camera, selected);
        {
        let mut fbo_mgr = self.resource.fbo_manager.borrow_mut();
        let fbo_sel = fbo_mgr.get_mut_or_panic(self.fbo_selected);
        fbo_sel.cgl_use();
        }
        for p in self.passes.values()
        {
            let not = p.draw_frame(
                &self.resource,
                load.clone()
                );

            not_loaded += not;
        }
        fbo::Fbo::cgl_use_end();

        self.clean_passes();
        println!("OOOOBBBB :::::::::::::: {}", objects.len());
        self.add_mmr(camera, objects);
        self.add_mmr(camera, cameras);

        {
            let mmr = self.grid.to_mmr();
            self.add_mmr(camera, vec![mmr].as_slice());

            let m = 40f64;
            self.camera_repere.transform.position = 
                vec::Vec3::new(
                    -self.camera_ortho.data.width/2f64 +m, 
                    -self.camera_ortho.data.height/2f64 +m, 
                    -10f64);
            self.camera_repere.transform.orientation = 
                camera.orientation.inverse();

            let cam = CameraIdMat::from_transform_camera(&self.camera_ortho);
            let mmr = self.camera_repere.to_mmr();
            self.add_mmr(&cam, vec![mmr].as_slice());
        }

        {
        //self.fbo_all.read().unwrap().cgl_use();
        //let fbo_all = &self.resource.fbo_manager.borrow().get_from_state(self.fbo_all);
        //fbo_all.write().unwrap().cgl_use();
        
        let mut fbo_mgr = self.resource.fbo_manager.borrow_mut();
        let fbo_all = fbo_mgr.get_mut_or_panic(self.fbo_all);
        fbo_all.cgl_use();
        for p in self.passes.values()
        {
            let not = p.draw_frame(
                &self.resource,
                load.clone()
                );

            not_loaded = not_loaded + not;
        }
        fbo::Fbo::cgl_use_end();
        }

        /*
        for p in self.passes.values()
        {
            p.draw_frame(
                self.mesh_manager.clone(),
                self.material_manager.clone(),
                self.shader_manager.clone(),
                self.texture_manager.clone(),
                self.fbo_manager.clone(),
                );
        }
        */


        let l = vec![self.quad_all.to_mmr()];
        self.prepare_passes_objects_ortho(&l);

        for p in self.passes.values()
        {
            let not = p.draw_frame(
                &self.resource,
                load.clone()
                );

            not_loaded = not_loaded + not;
        }

        let sel_len = selected.len();
        println!("selllll :::::::::::::: {}", sel_len);

        if sel_len > 0 {
            let l = vec![self.quad_outline.to_mmr()];
            self.prepare_passes_objects_ortho(&l);

            for p in self.passes.values()
            {
                let not = p.draw_frame(
                    &self.resource,
                    load.clone()
                    );
            
                not_loaded = not_loaded + not;
            }

            //* TODO dragger
            unsafe { render::cgl_clear(); }
            //ld.push_back(self.dragger.clone());
            //self.prepare_passes_objects_per(ld);
            //self.prepare_passes_objects_per(draggers);
            println!("dragger :::::::::::::: {}", draggers.len());
            self.prepare_passes_objects_per_mmr(camera, draggers);


            /*
            fn get_camera_resize_w(camera : &camera::Camera, m : &matrix::Matrix4, factor : f64) -> f64
            {
                let cam_mat = camera.object.read().unwrap().get_world_matrix();
                let projection = camera.get_perspective();

                let cam_mat_inv = cam_mat.get_inverse();
                let world_inv = &cam_mat_inv * m;

                let mut tm = &projection * &world_inv;
                tm = tm.transpose();

                let zero = vec::Vec4::new(0f64,0f64,0f64,1f64);
                let vw = &tm * zero;
                let w = vw.w * factor;
                return w;
            }


            
            let scale = get_camera_resize_w(&*self.camera.borrow(),
                &draggers.front().unwrap().read().unwrap().get_matrix(),
                0.05f64);
            //add_box(&mut *self.line.write().unwrap(), selected, scale as f32);
            add_box_only_first_object(&mut *self.line.write().unwrap(), draggers, scale);

            prepare_passes_object(
                self.line.clone(),
                &mut self.passes,
                self.material_manager.clone(),
                self.shader_manager.clone(),
                self.camera.clone());
             */

            let mmr = self.line.to_mmr();
            self.add_mmr(camera, vec![mmr].as_slice());

            for p in self.passes.values()
            {
                let not = p.draw_frame(
                    &self.resource,
                    load.clone()
                    );
                not_loaded = not_loaded + not;
            }
            //*/
        }

        on_finish(false);
        not_loaded
    }
}

pub struct GameRender
{
    passes : HashMap<String, Box<RenderPass>>, //TODO check

    resource: Rc<resource::ResourceGroup>,

        /*
    mesh_manager : Arc<RwLock<resource::ResourceManager<mesh::Mesh>>>,
    shader_manager : Arc<RwLock<resource::ResourceManager<shader::Shader>>>,
    texture_manager : Arc<RwLock<resource::ResourceManager<texture::Texture>>>,
    material_manager : Arc<RwLock<resource::ResourceManager<material::Material>>>,
    fbo_manager : Arc<RwLock<resource::ResourceManager<fbo::Fbo>>>,
    */

    //camera_ortho : Rc<RefCell<camera::Camera>>,
}

impl GameRender {

    //TODO remove dragger and put "view_objects"
    pub fn new(//factory: &mut factory::Factory,
               resource : Rc<resource::ResourceGroup>
               //dragger : Arc<RwLock<object::Object>>,
               ) -> GameRender
    {
        /*
        let camera_ortho = Rc::new(RefCell::new(factory.create_camera()));
        {
            let mut cam = camera_ortho.borrow_mut();
            cam.data.projection = camera::Projection::Orthographic;
            cam.pan(&vec::Vec3::new(0f64,0f64,50f64));
        }
        */

        /*
        let material_manager = Arc::new(RwLock::new(resource::ResourceManager::new()));
        let shader_manager = Arc::new(RwLock::new(resource::ResourceManager::new()));
        let fbo_manager = Arc::new(RwLock::new(resource::ResourceManager::new()));
        */

        let r = GameRender { 
            passes : HashMap::new(),
            //mesh_manager : factory.mesh_manager.clone(),
            /*
            mesh_manager : Arc::new(RwLock::new(resource::ResourceManager::new())),
            shader_manager : shader_manager.clone(),
            texture_manager : Arc::new(RwLock::new(resource::ResourceManager::new())),
            material_manager : material_manager.clone(),
            fbo_manager : fbo_manager,
            */
            resource : resource,
            //camera_ortho : camera_ortho,
        };

        r
    }

    pub fn init(&mut self)
    {
    }

    pub fn resize(&mut self, w : c_int, h : c_int)
    {
        //let mut cam_ortho = self.camera_ortho.borrow_mut();
        //cam_ortho.set_resolution(w, h);
    }

    fn prepare_passes_objects_per_mmr(
        &mut self,
        camera : &CameraIdMat,
        mmr : &[MatrixMeshRender])
    {
        let load = Arc::new(Mutex::new(0));
        for (_,p) in self.passes.iter_mut()
        {
            p.passes.clear();
        }

        self.add_mmr(camera, mmr);
    }

    fn add_mmr(
        &mut self,
        camera : &CameraIdMat,
        mmr : &[MatrixMeshRender])
    {
        let load = Arc::new(Mutex::new(0));

        for m in mmr {
            let pass = render::get_pass_from_mesh_render(
                &m.mr,
                &mut self.passes,
                &mut self.resource.material_manager.borrow_mut(),
                &mut self.resource.shader_manager.borrow_mut(),
                camera,
                load.clone()
                );

            if let Some(cam_pass) = pass {
                cam_pass.add_mmr(m.clone());
            }
        }
    }


    pub fn draw(
        &mut self,
        camera : &CameraIdMat,
        objects : &[MatrixMeshRender],
        loading : Arc<Mutex<usize>>
        ) -> bool
    {
        self.prepare_passes_objects_per_mmr(camera, objects);

        let mut not_yet_loaded = 0;
        for p in self.passes.values()
        {
            let r = p.draw_frame(&self.resource, loading.clone());
            not_yet_loaded += r;
        }

        not_yet_loaded > 0
    }
}

fn create_grid(m : &mut mesh::Mesh, num : i32, space : i32)
{
    //TODO make something better then using add_line
    //ie create the vec and then add the buffer

    let color = vec::Vec4::new(1f64,1f64,1f64,0.1f64);
    let xc = vec::Vec4::new(1.0f64,0.247f64,0.188f64,0.4f64);
    let zc = vec::Vec4::new(0f64,0.4745f64,1f64,0.4f64);

    for i in  -num..num {
        let p1 = vec::Vec3::new((i*space) as f64, 0f64, (-space*num) as f64);
        let p2 = vec::Vec3::new((i*space) as f64, 0f64, (space*num) as f64);
        let s = geometry::Segment::new(p1,p2);
        if i == 0 {
            m.add_line(s, zc);
        }
        else {
            m.add_line(s, color);
        }
    }

    for i in  -num..num {
        let p1 = vec::Vec3::new((-space*num) as f64, 0f64, (i*space) as f64);
        let p2 = vec::Vec3::new((space*num) as f64, 0f64, (i*space) as f64);
        let s = geometry::Segment::new(p1,p2);
        if i == 0 {
            m.add_line(s, xc);
        }
        else {
            m.add_line(s, color);
        }
    }
}

fn create_repere(m : &mut mesh::Mesh, len : f64)
{
    let red = vec::Vec4::new(1.0f64,0.247f64,0.188f64,1f64);
    let green = vec::Vec4::new(0.2117f64,0.949f64,0.4156f64,1f64);
    let blue = vec::Vec4::new(0f64,0.4745f64,1f64,1f64);

    let s = geometry::Segment::new(
        vec::Vec3::zero(), vec::Vec3::new(len, 0f64, 0f64));
    m.add_line(s, red);

    let s = geometry::Segment::new(
        vec::Vec3::zero(), vec::Vec3::new(0f64, len, 0f64));
    m.add_line(s, green);

    let s = geometry::Segment::new(
        vec::Vec3::zero(), vec::Vec3::new(0f64, 0f64, len));
    m.add_line(s, blue);
}

fn create_outline_material(
    rm : &mut resource::ResourceManager<material::Material>,
    sm : &mut resource::ResourceManager<shader::Shader>
    ) -> ResTT<material::Material>
{
    let outline_mat_name = "material/outline.mat";

    // 1st way is to read the json file
    //let outline_mat = rm.request_use_no_proc_tt_instance(outline_mat_name);

    //2nd way is write with json in code
    /*
    let json = r#"
{
  "name": "material/outline.mat",
  "shader": {
    "name": "shader/outline.sh"
  },
  "textures": {
    "texture": {
      "Fbo":
        [{
          "name": "fbo_selected"
        },
        "Depth"
        ]
    },
    "texture_all": {
      "Fbo": [
        {
          "name": "fbo_all"
        },
        "Depth"
      ]
    }
  },
  "uniforms": {
    "resolution": {
      "Vec2": {
        "x": 5.6,
        "y": 1.2
      }
    }
  }
}
"#;    

    use serde_json;
    let outline_mat : material::Material = serde_json::from_str(json).unwrap();
    */

    //3rd way is write by code
    let mut outline_mat = material::Material::new(outline_mat_name);
    outline_mat.shader = Some(create_outline_shader(sm));

    outline_mat.set_texture_data("texture", 
                                 material::Sampler::Fbo(resource::ResTT::new("fbo_selected"), fbo::Attachment::Depth));
    outline_mat.set_texture_data("texture_all", 
                                 material::Sampler::Fbo(resource::ResTT::new("fbo_all"), fbo::Attachment::Depth));

    //this is needed for the 2 and 3rd way
    let mut outline_mat_res = rm.add_resource(outline_mat_name, outline_mat);
    outline_mat_res.create_instance_with_manager(rm);

    outline_mat_res
}

fn create_all_material(
    rm : &mut resource::ResourceManager<material::Material>,
    sm : &mut resource::ResourceManager<shader::Shader>
    ) -> ResTT<material::Material>
{
    let mat_name = "material/fbo_all.mat";

    let mut mat = material::Material::new(mat_name);
    //mat.shader = Some(resource::ResTT::new("shader/simple_normal.sh"));
    mat.shader = Some(create_shader(sm, "shader/simple_normal.sh", SHADER_VERT_SIMPLE, SHADER_FRAG_SIMPLE_NORMAL));

    mat.set_texture_data("texture", 
                         material::Sampler::Fbo(resource::ResTT::new("fbo_all"), fbo::Attachment::Color));

    let mut mat_res = rm.add_resource(mat_name, mat);
    mat_res.create_instance_with_manager(rm);

    mat_res
}

fn create_line_material(
    rm : &mut resource::ResourceManager<material::Material>,
    sm : &mut resource::ResourceManager<shader::Shader>
    ) -> ResTT<material::Material>
{
    let mat_name = "material/line.mat";

    let mut mat = material::Material::new(mat_name);
    //mat.shader = Some(resource::ResTT::new("shader/line.sh"));
    mat.shader = Some(create_shader(sm, "shader/line.sh", SHADER_VERT_LINE, SHADER_FRAG_LINE));

    let mat_res = rm.add_resource(mat_name, mat);

    mat_res
}

fn create_outline_shader(rm : &mut resource::ResourceManager<shader::Shader>) -> ResTT<shader::Shader>
{
    //resource::ResTT::new("shader/outline.sh"));
    create_shader(rm, "shader/outline.sh", SHADER_VERT_SIMPLE, SHADER_FRAG_OUTLINE)
}

fn create_shader(
    rm : &mut resource::ResourceManager<shader::Shader>,
    name : &str,
    vert : &str,
    frag : &str
    ) -> ResTT<shader::Shader>
{

    let s = shader::Shader::with_vert_frag(name.to_owned(), vert.to_owned(), frag.to_owned());
    let res = rm.add_resource(name, s);

    res
}

static SHADER_VERT_SIMPLE : &'static str = r#"
attribute vec3 position;
attribute vec2 texcoord;
uniform mat4 matrix;

varying vec2 f_texcoord;

void main(void)
{
  gl_Position = matrix * vec4(position, 1.0);
  f_texcoord = texcoord;
}

"#;

static SHADER_FRAG_OUTLINE : &'static str = r#"
#ifdef GL_ES
precision highp float;
#endif
uniform sampler2D texture;
uniform sampler2D texture_all;
uniform vec2 resolution;

void main (void)
{
  vec4 tz = texture2D(texture, vec2(gl_FragCoord.x/resolution.x,gl_FragCoord.y/resolution.y));

  vec4 allz = texture2D(texture_all, vec2(gl_FragCoord.x/resolution.x,gl_FragCoord.y/resolution.y));
  vec4 tz1 = texture2D(texture, vec2((gl_FragCoord.x+1.0)/resolution.x,gl_FragCoord.y/resolution.y));
  vec4 tz2 = texture2D(texture, vec2((gl_FragCoord.x-1.0)/resolution.x,gl_FragCoord.y/resolution.y));
  vec4 tz3 = texture2D(texture, vec2(gl_FragCoord.x/resolution.x,(gl_FragCoord.y+1.0)/resolution.y));
  vec4 tz4 = texture2D(texture, vec2(gl_FragCoord.x/resolution.x,(gl_FragCoord.y-1.0)/resolution.y));

  float depth = tz1.r;
  if (depth > tz2.r)
    depth = tz2.r;
  if (depth > tz3.r)
    depth = tz3.r;
  if (depth > tz4.r)
    depth = tz4.r;

  if (tz.r == 1.0 &&
        depth != 1.0
     ) {

    if (allz.r > depth)
    gl_FragColor = vec4(1,0,0,1);
    else
    gl_FragColor = vec4(0.5,0,0,0.5);
  }
  else
  gl_FragColor = vec4(0,0,0,0.0);

  //gl_FragColor = allz;
  //gl_FragColor = vec4(0,0,1,1);
}
"#;

static SHADER_FRAG_SIMPLE_NORMAL : &'static str = r#"
#ifdef GL_ES
precision highp float;
#endif
uniform vec4 color;
uniform sampler2D texture;
varying vec2 f_texcoord;

void main (void)
{
  vec2 texc = f_texcoord;
  vec4 diffuse_tex = texture2D(texture, texc);
  if (color.x < -1.5)
  {
  gl_FragColor = vec4(0.000003*texc.x + color.x, color.y, color.z, 1.0);
  }
  else {
  gl_FragColor = diffuse_tex;
  }

  //gl_FragColor = vec4(0.3, 0.3, 0.4, 1.0);
  //gl_FragColor = vec4(red, 0.3, 0.4, 1.0);
  //vec4 ccolor = vec4(color.x, 0.3, 0.4, 1.0);
  //gl_FragColor = color;
}
"#;

static SHADER_VERT_LINE : &'static str = r#"
attribute vec3 position;
attribute vec4 color;
uniform mat4 matrix;
uniform int size_fixed;

varying vec4 vcolor;

void main(void)
{
  vcolor = color;

  if (size_fixed == 1) {
    // change value to match resolution.    = (2 * ObjectSizeOnscreenInPixels / ScreenWidthInPixels)
    // transform the vector (0,0,0) to clipspace.
    // This will get the W the object's pivot will be divided by, thus you get the "inverseScale" that will be applied to vertices.

    float reciprScaleOnscreen = 0.1;

    float w = (matrix * vec4(0,0,0,1)).w;
    w *= reciprScaleOnscreen;

    gl_Position = matrix * vec4(position.xyz * w , 1);
  }
  else {
    gl_Position = matrix * vec4(position, 1.0);
  }

}

"#;

static SHADER_FRAG_LINE : &'static str = r#"
#ifdef GL_ES
precision highp float;
#endif
varying vec4 vcolor;

void main (void)
{
  //gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
  gl_FragColor = vcolor;
}

"#;
