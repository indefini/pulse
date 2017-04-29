use std::rc::Rc;
use std::cell::{RefCell, BorrowState};
use std::any::{Any};//, AnyRefExt};
use std::f64::consts;

use dormin::transform;
use dormin::camera;
use context;
use dragger;
use dormin::intersection;
use dormin::vec;
use dormin::resource;
use ui;

use util;


pub enum State
{
    Idle,
    CameraRotation,
    Dragger,
    MultipleSelect
}

pub struct Control
{
    pub camera : Rc<RefCell<camera::Camera>>,
    state : State,
    dragger : Rc<RefCell<dragger::DraggerManager>>,
    mouse_start : Option<vec::Vec2>,
    resource : Rc<resource::ResourceGroup>
}

impl Control
{
    pub fn new(
        camera : Rc<RefCell<camera::Camera>>,
        dragger : Rc<RefCell<dragger::DraggerManager>>,
        resource : Rc<resource::ResourceGroup>
        ) -> Control
    {
        Control {
            //factory : Rc::new(RefCell::new(factory::Factory::new())),
            camera : camera,
            //tree : None,
            state : State::Idle,
            dragger : dragger,
            mouse_start : None,
            resource : resource
        }
    }

    pub fn mouse_down(
            &mut self,
            context : &context::ContextOld,
            modifier : i32,
            button : i32,
            x : i32,
            y : i32,
            timestamp : i32) -> Vec<ui::EventOld>
    {
        let mut v = Vec::new();

        if (modifier & 1) != 0 {
            println!("pressed shift");
        }
        else if modifier & (1 << 1) != 0 {
            self.mouse_start = Some(vec::Vec2::new(x as f64, y as f64));
            self.state = State::MultipleSelect;
            v.push(ui::Event::RectVisibleSet(true));
            v.push(ui::Event::RectSet(x as f32, y as f32, 1f32, 1f32));
            println!("pressed control");
            return v;
        }

        let objs = context.selected.clone();
        if !objs.is_empty() {
            let click = self.dragger.borrow_mut().mouse_down(
                &*self.camera.borrow(),button, x, y, &*self.resource);
            if click {
                self.state = State::Dragger;
                v.push(ui::Event::DraggerClicked);
            }
        }

        return v;
    }

    pub fn mouse_up(
            &mut self,
            context : &context::ContextOld,
            button : i32,
            x : i32,
            y : i32,
            timestamp : i32) -> ui::EventOld
    {
        match self.state {
            State::CameraRotation => {
                self.state = State::Idle;
                println!("state was cam rotate ");
                return ui::Event::Empty;
            },
            State::Dragger => {
                self.state = State::Idle;
                let o = self.dragger.borrow_mut().mouse_up(
                    &*self.camera.borrow(),
                    button,
                    x,
                    y);

                if let Some(op) = o {
                    return ui::Event::DraggerOperation(op);
                }
                return ui::Event::Empty;
            },
            State::MultipleSelect => {
                self.state = State::Idle;
                return ui::Event::RectVisibleSet(false);
            },
            _ => {}
        }

        //println!("rust mouse up button {}, pos: {}, {}", button, x, y);
        let r = match self.camera.borrow_state(){
            BorrowState::Writing => {
                println!("cannot borrow camera");
                return ui::Event::Empty;
            },
            _ => {
                self.camera.borrow().ray_from_screen(x as f64, y as f64, 10000f64)
            }
        };

        /*
        let scene = match self.context.borrow_state(){
            BorrowState::Writing => { println!("cannot borrow context"); return operation::Change::None; }
            _ => {
                let c = self.context.borrow();
                let scene = match c.scene {
                    Some(ref s) => s.clone(),
                    None => {
                        println!("no scene ");
                        return operation::Change::None;
                    }
                };
                scene
            }
        };
        */
        let scene = match context.scene {
             Some(ref s) => s.clone(),
             None => {
                 println!("no scene ");
                 return ui::Event::Empty;
             }
        };

        //TODO
        println!("TODO dont test all objects in the scene, but only visible ones : {}", scene.borrow().objects.len());

        let mut found_length = 0f64;
        let mut closest_obj = None;

        //TODO collision for cameras.
        {
        let mut mesh_manager = self.resource.mesh_manager.borrow_mut();
        let cam_mesh = mesh_manager.get_or_create("model/camera.mesh");
        for c in &scene.borrow().cameras {
            println!("testing camera {}", c.borrow().id);
            let cam = c.borrow();
            let o = cam.object.read().unwrap();
            let pos = o.world_position();
            let ori = o.world_orientation();
            let cam_scale = get_camera_scale(&*self.camera.borrow(), &o.get_world_matrix());
            let scale = o.world_scale() * cam_scale;
            let ir = intersection::ray_mesh(&r, cam_mesh, &pos, &ori, &scale);
            if ir.hit {
                println!("camera collision!!!!");
                let length = (ir.position - r.start).length2();
                /*
                match closest_obj {
                    None => {
                        closest_obj = Some(o.clone());
                        found_length = length;
                    }
                    Some(_) => {
                        if length < found_length {
                            closest_obj = Some(o.clone());
                            found_length = length;
                        }
                    }
                }
                */
            }
        }
        }

        for o in &scene.borrow().objects {
            let ir = intersection::ray_object(&r, &*o.read().unwrap(), &*self.resource);
            if ir.hit {
                let length = (ir.position - r.start).length2();
                match closest_obj {
                    None => {
                        closest_obj = Some(o.clone());
                        found_length = length;
                    }
                    Some(_) => {
                        if length < found_length {
                            closest_obj = Some(o.clone());
                            found_length = length;
                        }
                    }
                }
            }
        }

        let mut v = Vec::new();
        match closest_obj {
            None => {},
            Some(o) => v.push(o)
        }

        return ui::Event::ChangeSelected(v);
    }

    fn rotate_camera(&mut self, context : &context::ContextOld, x : f64, y : f64)
    {
        self.state = State::CameraRotation;

        let mut camera = self.camera.borrow_mut();
        let cori = camera.object.read().unwrap().orientation;

        let (result, angle_x, angle_y) = {
            let cam = &mut camera.data;

            if vec::Vec3::up().dot(&cori.rotate_vec3(&vec::Vec3::up())) <0f64 {
                cam.yaw = cam.yaw + 0.005*x;
            }
            else {
                cam.yaw = cam.yaw - 0.005*x;
            }

            cam.pitch -= 0.005*y;

            let qy = vec::Quat::new_axis_angle_rad(vec::Vec3::up(), cam.yaw);
            let qp = vec::Quat::new_axis_angle_rad(vec::Vec3::right(), cam.pitch);

            (
                qy * qp,
                cam.pitch/consts::PI*180f64,
                cam.yaw/consts::PI*180f64,
                )
        };

        if !context.selected.is_empty() {
            let center = util::objects_center(&context.selected);
            camera.set_center(&center);
        }

        camera.rotate_around_center(&result);

        let mut c = camera.object.write().unwrap();
        //(*c).orientation = vec::Quat::new_yaw_pitch_roll_deg(angle_y, angle_x, 0f64);
        (*c).orientation = transform::Orientation::Quat(vec::Quat::new_yaw_pitch_roll_deg(angle_y, angle_x, 0f64));
        //self.state = CameraRotation;
    }

    pub fn mouse_move(
        &mut self,
        context : &context::ContextOld,
        mod_flag : i32,
        button : i32,
        curx : i32,
        cury : i32,
        prevx : i32,
        prevy : i32,
        timestamp : i32) -> Vec<ui::EventOld>
    {
        let mut list = Vec::new();

        match self.state {
            State::Idle | State::CameraRotation => {
                let x : f64 = curx as f64;
                let y : f64 = cury as f64;

                let r = match self.camera.borrow_state(){
                    BorrowState::Writing => {
                        println!("cannot borrow camera");
                        return list;
                    },
                    _ => {
                        self.camera.borrow().ray_from_screen(x as f64, y as f64, 10000f64)
                    }
                };

                let update =
                    self.dragger.borrow_mut().mouse_move_hover(r, button, &*self.resource) || button == 1;

                if button == 1 {

                    self.dragger.borrow_mut().set_state(dragger::State::Idle);

                    let x : f64 = curx as f64 - prevx as f64;
                    let y : f64 = cury as f64 - prevy as f64;

                    if (mod_flag & 1) != 0 {
                        let t = vec::Vec3::new(-x*0.5f64, y*0.5f64, 0f64);
                        let mut camera = self.camera.borrow_mut();
                        camera.pan(&t);
                    }
                    else {
                        self.rotate_camera(context, x, y);
                        //let camera = self.camera.borrow();
                        println!("remove from update and move here");
                        //self.dragger.borrow_mut().set_orienation(&*camera);
                    }
                }

                if update {
                    list.push(ui::Event::CameraChange);
                }
            },
            State::Dragger =>
            {
                let camera_clone = self.camera.clone();
                let camera = match camera_clone.borrow_state(){
                    BorrowState::Writing => {
                        println!("cannot borrow camera");
                        return list;
                    },
                    _ => camera_clone.borrow(),
                };

                let x : f64 = curx as f64;// - prevx as f64;
                let y : f64 = cury as f64;// - prevy as f64;
                let opsome = self.dragger.borrow_mut().mouse_move(&*camera,x,y);
                if let Some(op) = opsome {
                    match op {
                        dragger::Operation::Translation(v) => {
                            //list.push_back(self.request_translation(v));
                            list.push(ui::Event::DraggerTranslation(v));
                        },
                        dragger::Operation::Scale(v) => {
                            //list.push_back(self.request_scale(v));
                            list.push(ui::Event::DraggerScale(v));
                        },
                        dragger::Operation::Rotation(q) => {
                            //list.push_back(self.request_rotation(q));
                            list.push(ui::Event::DraggerRotation(q));
                        }
                    }
                }
            }
            State::MultipleSelect => {
                if let Some(ms) = self.mouse_start {
                    let x = curx as f32;
                    let y = cury as f32;
                    let ex = ms.x as f32;
                    let ey = ms.y as f32;
                    let (startx, endx) = if x < ex {(x, ex - x)} else {(ex, x - ex)};
                    let (starty, endy) = if y < ey {(y, ey - y)} else {(ey, y - ey)};
                    list.push(ui::Event::RectSet(startx, starty, endx, endy));

                    let planes = self.camera.borrow().get_frustum_planes_rect(
                        startx as f64,
                        starty as f64,
                        endx as f64,
                        endy as f64);

                    let s = match context.scene {
                        Some(ref s) => s.clone(),
                        None => return list
                    };

                    let mut obvec = Vec::new();
                    let mut has_changed = false;
                    for o in &s.borrow().objects {
                        let b = intersection::is_object_in_planes(planes.as_ref(), &*o.read().unwrap(), &*self.resource);
                        if b {
                            if !context.has_object(o.clone()) {
                                has_changed = true;
                            }
                            obvec.push(o.clone());
                        }
                    }

                    if !has_changed {
                        if context.selected.len() != obvec.len() {
                                has_changed = true;
                        }
                    }

                    if has_changed {
                        list.push(ui::Event::ChangeSelected(obvec));
                    }

                }
            }
        }

        return list;
    }

    pub fn mouse_wheel(
        &self,
        modifier : i32,
        direction : i32,
        z : i32,
        x : i32,
        y : i32,
        timestamp : i32
        )
    {
        let mut axis = vec::Vec3::new(0f64, 0f64, z as f64);
        //axis = axis * 10.5f64;
        axis = axis * 0.2f64;
        let mut camera = self.camera.borrow_mut();
        camera.pan(&axis);
    }

    pub fn key_down(
        &mut self,
        modifier : i32,
        keyname : &str,
        key : &str,
        timestamp : i32
        ) ->  ui::EventOld
    {
        let mut t = vec::Vec3::zero();

        match key {
            "e" => t.z = -50f64,
            "d" => t.z = 50f64,
            "f" => t.x = 50f64,
            "s" => t.x = -50f64,
            "z" => {
                return ui::Event::Undo;
            },
            "r" => {
                return ui::Event::Redo;
            },
            "space" => {
                self.dragger.borrow_mut().change();
            },
            _ => {
                println!("key not implemented : {}", key);
            }
        }

        {
            let mut camera = self.camera.borrow_mut();
            let p = camera.object.read().unwrap().position;
            camera.object.write().unwrap().position = p + t;
        }

        return ui::Event::DraggerChange;
    }
}

pub trait WidgetUpdate {

    //fn update_changed<T : Any+Clone>(
    fn update_changed(
        &mut self,
        name : &str,
        //old : Option<&T>,
        new : &Any);
}


use dormin::matrix;
fn get_camera_scale(camera : &camera::Camera, world_matrix : &matrix::Matrix4) -> f64
{
    let cam_mat = camera.object.read().unwrap().get_world_matrix();
    let projection = camera.get_perspective();
    let cam_mat_inv = cam_mat.get_inverse();

    let world_inv = &cam_mat_inv * world_matrix;

    let mut tm = &projection * &world_inv;
    tm = tm.transpose();

    let zero = vec::Vec4::new(0f64,0f64,0f64,1f64);
    let vw = &tm * zero;
    let factor = 0.05f64;
    let w = vw.w * factor;
    w
}
