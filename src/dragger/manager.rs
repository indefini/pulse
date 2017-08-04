use dormin::mesh;
use dormin::vec;
use dormin::resource;
use dormin::shader;
use dormin::material;
use dormin::transform;
use dormin::mesh_render;
use dormin::geometry;
use dormin::intersection;
use dormin::matrix;
use dormin::camera2;
use dormin::render::MatrixMeshRender;
use uuid;

use dragger::{
    TranslationMove,
    create_dragger_translation_group
};
use dragger::{
    ScaleOperation,
    create_scale_draggers
};
use dragger::{
    RotationOperation,
    create_rotation_draggers
};

pub type DraggerGroup = Vec<Dragger>;

pub struct DraggerManager
{
    draggers : Vec<DraggerGroup>,
    mouse_start : vec::Vec2,
    mouse : Option<Box<DraggerMouse+'static>>,
    ori : vec::Quat,
    current_group : usize,
    dragger_focus : Option<uuid::Uuid>
}

#[derive(Copy,Clone,Debug)]
pub enum State
{
    Idle,
    Highlight,
    Selected,
    LowLight,
    Hide,
    ShowSecond
}

pub enum Kind
{
    Translate,
    Scale,
    Rotate
}

pub enum Operation
{
    Translation(vec::Vec3),
    Scale(vec::Vec3),
    Rotation(vec::Quat)
}

#[derive(Copy,Clone)]
pub enum Repere
{
    Global,
    Local
}

pub enum Collision
{
    MeshAABox,
    Mesh,
    SpecialMesh(resource::ResTT<mesh::Mesh>)
}

#[derive(Clone, Debug)]
pub struct TransformMeshRender
{
    pub transform : transform::Transform,
    pub mesh_render : mesh_render::MeshRender
}

impl TransformMeshRender {
    pub fn new(t : transform::Transform, mr : mesh_render::MeshRender) -> TransformMeshRender
    {
        TransformMeshRender {
        transform : t,
        mesh_render : mr
        }
    }
}

pub struct Dragger
{
    object : TransformMeshRender,
    pub ori : transform::Orientation,
    pub constraint : vec::Vec3,
    kind : Kind,
    color : vec::Vec4,
    repere : Repere,
    collision : Collision,
    state : State,
    scale : f64,
    id : uuid::Uuid,
}

impl DraggerManager
{
    pub fn new() -> DraggerManager
    {
        let mut dm = DraggerManager {
            draggers : Vec::with_capacity(3),
            mouse_start : vec::Vec2::zero(),
            mouse : None,
            ori : vec::Quat::identity(),
            current_group : 0usize,
            dragger_focus : None
        };

        let tr = create_dragger_translation_group();
        dm.draggers.push(tr);

        let sc = create_rotation_draggers();
        dm.draggers.push(sc);

        let sc = create_scale_draggers();
        dm.draggers.push(sc);

        dm
    }

    pub fn mouse_down(
        &mut self,
        c : &camera2::CameraTransform,
        button : i32,
        x : i32,
        y : i32,
        resource : &resource::ResourceGroup
        ) -> bool
    {
        self.mouse_start.x = x as f64;
        self.mouse_start.y = y as f64;
        let r = c.ray_from_screen(x as f64, y as f64, 10000f64);

        self.check_collision(r, button, resource).is_some()
    }

    pub fn mouse_up(&mut self, c : &camera2::CameraTransform, button : i32, x : i32, y : i32)
        -> Option<Operation>
    {
        self.set_state(State::Idle);

        let op = if let Some(ref m) = self.mouse {
            m.mouse_move(
                c,
                self.mouse_start,
                vec::Vec2::new(x as f64, y as f64))
        }
        else  {
            return None;
        };

        self.mouse = None;
        return op;
    }

    pub fn check_collision(
        &mut self,
        r: geometry::Ray,
        button : i32,
        resource : &resource::ResourceGroup) -> Option<uuid::Uuid>
    {
        let mut found_length = 0f64;
        let mut closest_dragger = None;
        for (i, dragger) in self.draggers[self.current_group].iter_mut().enumerate() {
            dragger.set_state(State::Idle);
            let (hit, len) = dragger.check_collision(&r, dragger.scale, resource);
            if hit {
                if let None = closest_dragger {
                    closest_dragger = Some(i);
                    found_length = len;
                }
                else if len < found_length {
                    closest_dragger = Some(i);
                    found_length = len;
                }
            }
        }

        if let Some(i) = closest_dragger {
            let mut dragger = &mut self.draggers[self.current_group][i];
            match button {
                0i32 => dragger.set_state(State::Highlight),
                1i32 => {
                    dragger.set_state(State::Selected);
                    match dragger.kind {
                        Kind::Translate => {
                            //println!("ori : {:?}", self.ori);

                            self.mouse = Some(box TranslationMove::new(
                                    dragger.object.transform.position,
                                    dragger.constraint,
                                    dragger.repere,
                                    self.ori
                                    ) as Box<DraggerMouse>);
                        }
                        Kind::Scale => {
                            self.mouse = Some(box ScaleOperation::new(
                                    dragger.object.transform.position,
                                    dragger.constraint,
                                    dragger.repere,
                                    self.ori
                                    ) as Box<DraggerMouse>);
                        }
                        Kind::Rotate => {
                            //println!("ori : {:?}", self.ori);

                            self.mouse = Some(box RotationOperation::new(
                                    dragger.object.transform.position,
                                    dragger.constraint,
                                    dragger.repere,
                                    self.ori
                                    ) as Box<DraggerMouse>);
                        }
                    }
                }
                _ => {}
            };
            Some(dragger.id)
        }
        else {
            None
        }
    }

    pub fn mouse_move_hover(
        &mut self,
        r: geometry::Ray,
        button : i32,
        resource : &resource::ResourceGroup) -> bool
    {
        let result = self.check_collision(r, button, resource);

        if let Some(id) = result {
            if let Some(focus) = self.dragger_focus {
                if focus == id {
                    return false;
                }
            }
        }
        else {
            if self.dragger_focus.is_none(){
                return false;
            }
        }

        self.dragger_focus = result;
        true
    }

    pub fn set_position(&mut self, p : vec::Vec3) {
        for d in &mut self.draggers[self.current_group] {
            d.object.transform.position = p;
        }

    }

    pub fn set_orientation(&mut self, ori : transform::Orientation, camera_position : &vec::Vec3) {
        self.ori = ori.as_quat();
        for d in &mut self.draggers[self.current_group] {
            if self.current_group == 2usize {
                d.face_camera(camera_position, self.ori);
            }
            else {
                d.object.transform.orientation = ori * d.ori;
            }
        }
    }

    pub fn scale_to_camera(&mut self,
        cam_mat_inv : &matrix::Matrix4,
        projection : &matrix::Matrix4)
    {
        for d in &mut self.draggers[self.current_group] {

            d.scale_to_camera_data(&cam_mat_inv, &projection);
        }
    }

    pub fn get_mmr(&mut self) -> Vec<MatrixMeshRender>
    {
        let mut l = Vec::new();
        for d in &mut self.draggers[self.current_group] {
            d.object.transform.set_as_dirty();
            let mat = d.object.transform.get_or_compute_local_matrix().clone();
            let mmr = MatrixMeshRender::new(mat, d.object.mesh_render.clone());
            l.push(mmr);
        }

        l
    }

    pub fn set_state(&mut self, state : State) {
        for d in &mut self.draggers[self.current_group] {
            d.set_state(state);
        }
    }

    pub fn mouse_move(
        &mut self,
        camera : &camera2::CameraTransform,
        cur_x : f64,
        cur_y : f64) -> Option<Operation>
    {
        if let Some(ref m) = self.mouse {
            return m.mouse_move(
                camera,
                self.mouse_start,
                vec::Vec2::new(cur_x, cur_y));
        }

        None
    }

    pub fn change(&mut self)
    {
        let mut newlen = self.current_group + 1;
        if newlen >= self.draggers.len() {
            newlen = 0;
        }

        self.current_group = newlen;
    }

}

pub fn create_dragger(
    name : &str,
    mesh : &str,
    color : vec::Vec4) -> TransformMeshRender
{
    let mat = create_mat(color, name);

    let mr = mesh_render::MeshRender::new_with_mat2(
        mesh,
        mat,
        );

    TransformMeshRender::new(Default::default(), mr)
}

fn create_mat(color : vec::Vec4, name : &str) -> material::Material
{
    //let mut mat = material::Material::new_from_file("material/dragger.mat");
    let mut mat = material::Material::new("material/dragger.mat");
    /*
    let shader = shader::Shader::with_vert_frag(
        "shader/dragger.sh".to_owned(),
        SHADER_VERT.to_owned(),
        SHADER_FRAG.to_owned());

    mat.shader = Some(resource::ResTT::new_with_instance("shader/dragger.sh", shader));
    */
    mat.shader = Some(resource::ResTT::new("shader/dragger.sh"));

    mat.set_uniform_data(
        "color",
        shader::UniformData::Vec4(color));

    mat
}

impl Dragger
{
    pub fn new(
        object : TransformMeshRender,
        constraint : vec::Vec3,
        ori : transform::Orientation,
        kind : Kind,
        color : vec::Vec4,
        collision : Collision
        ) -> Dragger
    {
        Dragger {
            object : object,
            constraint : constraint,
            ori : ori,
            kind : kind,
            color : color,
            repere : Repere::Local,
            collision : collision,
            state : State::Idle,
            scale : 1f64,
            id : uuid::Uuid::new_v4(),
        }
    }

    pub fn update_scale(
        &mut self,
        world : &matrix::Matrix4,
        projection : &matrix::Matrix4)
    {
        
    }
}

impl Dragger
{
    //TODO
    // We change object transform and object material (color)
    
    fn scale_to_camera_data(
        &mut self,
        cam_mat_inv : &matrix::Matrix4,
        projection : &matrix::Matrix4)
    {
        self.object.transform.set_as_dirty();
        let world_inv = cam_mat_inv * self.object.transform.get_or_compute_local_matrix();

        let mut tm = projection * &world_inv;
        tm = tm.transpose();

        let zero = vec::Vec4::new(0f64,0f64,0f64,1f64);
        let vw = &tm * zero;
        let factor = 0.05f64;
        let w = vw.w * factor;
        self.scale = w;
    }

    fn set_state(&mut self, state : State)
    {
        fn set_color(s : &mut Dragger, color : vec::Vec4){
            //TODO material instance
            if let Some(ref mut mat) = s.object.mesh_render.material.get_instance() {
                mat.set_uniform_data(
                    "color",
                    shader::UniformData::Vec4(color));
            }
        }


        match state {
            State::Highlight => {
                set_color(self, vec::Vec4::new(1f64,1f64,0f64, 1f64));
            },
            State::Selected => {
                set_color(self, vec::Vec4::new(1f64,1f64,1f64, 1f64));
            },
            State::Idle => {
                let c = self.color;
                set_color(self, c);
            }
            _ => {}
        }

        self.state = state;
    }

    fn check_collision(&self, r : &geometry::Ray, s : f64, resource : &resource::ResourceGroup) -> (bool, f64)
    {
        let ob = &self.object;
        let position = ob.transform.position;
        let rotation = ob.transform.orientation.as_quat();
        let scale = vec::Vec3::new(s, s, s);

        let mut mm = resource.mesh_manager.borrow_mut();

        let ir = 
        {
            let mesh = match ob.mesh_render.mesh.get_or_load_ref(&mut mm) {
                None => {
                    return (false,0f64);
                },
                Some(m) => m
            };

            let aabox = match mesh.aabox {
                None => {
                    return (false,0f64);
                },
                Some(ref aa) => aa
            };

            intersection::intersection_ray_box(r, aabox, &position, &rotation, &scale)
        };

        if ir.hit {
            let length = (ir.position - r.start).length2();

            //TODO 

            let m = match self.collision{
                Collision::SpecialMesh(ref r) => {
                    if let Some(m) = r.as_ref_instant(&mut *mm) {
                        m
                    }
                    else {
                        return (true, length);
                    }
                },
                _ => return (true, length)
            };

            let ir = intersection::ray_mesh(r, m, &position, &rotation, &scale);
            if ir.hit {
                let length = (ir.position - r.start).length2();
                return (true, length);
            }
            else {
                return (false, 0f64);
            }
        }
        else {
            return (false,0f64);
        }
    }

    pub fn face_camera(
        &mut self,
        camera_position : &vec::Vec3,
        manager_ori : vec::Quat,
        )
    {
        let qo = manager_ori;
        let mut o = &mut self.object;
        let constraint = self.constraint;
        let dragger_ori = self.ori.as_quat();

        let diff = o.transform.position - *camera_position;
        let dotx = diff.dot(&qo.rotate_vec3(&vec::Vec3::x()));
        let doty = diff.dot(&qo.rotate_vec3(&vec::Vec3::y()));
        let dotz = diff.dot(&qo.rotate_vec3(&vec::Vec3::z()));
        let mut angle = 0f64;

        if constraint ==  vec::Vec3::new(0f64,0f64,1f64) {
            if dotx > 0f64 {
                if doty > 0f64 {
                    angle = 180f64;
                }
                else {
                    angle = 90f64;
                }
            }
            else if doty > 0f64 {
                angle = -90f64;
            }
        }

        if constraint == vec::Vec3::new(0f64,1f64,0f64) {
            if dotx > 0f64 {
                if dotz > 0f64 {
                    angle = 180f64;
                }
                else {
                    angle = 90f64;
                }
            }
            else if dotz > 0f64 {
                angle = -90f64;
            }
        }

        if constraint == vec::Vec3::new(1f64,0f64,0f64) {
            if doty > 0f64 {
                if dotz > 0f64 {
                    angle = -180f64;
                }
                else {
                    angle = -90f64;
                }
            }
            else if dotz > 0f64 {
                angle = 90f64;
            }
        }


        let q = vec::Quat::new_yaw_pitch_roll_deg(0f64,0f64, angle);
        let qoo = dragger_ori *q;
        let qf = qo * qoo;

        o.transform.orientation = transform::Orientation::Quat(qf);
    }

}

pub trait DraggerMouse
{
    fn mouse_move(
        &self,
        camera : &camera2::CameraTransform,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2)
        -> Option<Operation>;
}

pub static SHADER_VERT : &'static str = r#"
attribute vec3 position;
uniform mat4 matrix;

void main(void)
{
    float reciprScaleOnscreen = 0.05;

    float w = (matrix * vec4(0,0,0,1)).w;
    w *= reciprScaleOnscreen;

    gl_Position = matrix * vec4(position.xyz * w , 1);

  //gl_Position = matrix * vec4(position, 1.0);
}

"#;

pub static SHADER_FRAG : &'static str = r#"
#ifdef GL_ES
precision highp float;
#endif
uniform vec4 color;

void main (void)
{
  gl_FragColor = color;
}

"#;

