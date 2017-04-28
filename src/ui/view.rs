use std::rc::Rc;
use std::cell::{Cell,RefCell, BorrowState};
use std::sync::{RwLock, Arc,Mutex};
use libc::{c_char, c_void, c_int, c_uint, c_float};
use std::ffi::CStr;
use std::str;
use uuid;
use dormin::object;
use dormin::shader;
use dormin::transform;

use ui;
use dormin::render::{Render, GameRender};
use dormin::resource;
use dormin::vec;
use dormin::material;
use dragger;
use dormin::camera;
use operation;
use control;
use control::Control;
use control::WidgetUpdate;
use dormin::scene;
use dormin::component;
use dormin::component::mesh_render;
use util;
use util::Arw;
use dormin::input;
use context;


#[link(name = "joker")]
extern {
    pub fn window_rect_visible_set(win :*const ui::Window, b : bool);
    pub fn window_rect_set(
        win :*const ui::Window,
        x : c_float,
        y : c_float,
        w : c_float,
        h : c_float);
}

pub struct View
{
    render : Box<Render>,
    pub control : Rc<RefCell<Control>>,

    pub window : Option<*const ui::Window>,

    dragger : Rc<RefCell<dragger::DraggerManager>>,

    pub camera : Rc<RefCell<camera::Camera>>,
    pub resource : Rc<resource::ResourceGroup>,
    pub uuid : uuid::Uuid,

    pub width : i32,
    pub height : i32,
    pub updating : Cell<bool>,
    loading_resource : Arc<Mutex<usize>>,
}

impl View
{
    pub fn new(
        resource : Rc<resource::ResourceGroup>,
        dragger : Rc<RefCell<dragger::DraggerManager>>,
        render : Box<Render>,
        w : i32,
        h : i32,
        camera : Rc<RefCell<camera::Camera>>
        ) -> View
    {
        let control = Rc::new(RefCell::new(
                Control::new(
                    camera.clone(),
                    dragger.clone(),
                    resource.clone(),
                    )));

        let v = View {
            render : render,
            control : control,

            window : None,

            dragger : dragger,

            camera : camera,
            resource : resource,
            uuid : uuid::Uuid::new_v4(),

            width : w,
            height : h,
            updating : Cell::new(false),
            loading_resource : Arc::new(Mutex::new(0)),
        };

        return v;
    }

    pub fn init(
        &mut self,
        win : *const ui::Window,
        ) {

        self.window = Some(win);
    }

    fn init_render(&mut self)
    {
        self.render.init();
    }

    fn draw(&mut self, context : &context::ContextOld) -> bool
    {
        let scene = match context.scene {
            Some(ref s) => s.borrow(),
            None => return false
        };

        let obs = &scene.objects;
        let sel = &context.selected;

        let mut center = vec::Vec3::zero();
        let mut ori = vec::Quat::identity();
        for o in sel {
            center = center + o.read().unwrap().world_position();
            ori = ori * o.read().unwrap().world_orientation();
        }

        if !sel.is_empty() {
            center = center / (sel.len() as f64);

            //TODO println!("remove this code from here, put in update or when moving the camera");
            let mut dragger = self.dragger.borrow_mut();
            dragger.set_position(center);
            dragger.set_orientation(transform::Orientation::Quat(ori), &*self.camera.borrow());
            //let scale = self.camera.borrow().get_camera_resize_w(0.05f64);
            //dragger.set_scale(scale);
            dragger.scale_to_camera(&*self.camera.borrow());
        }

        let finish = |b| {
            //println!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~render finished");
        };

        let mut cams = Vec::new();
        for c in &scene.cameras {

            let mut camo = create_camera_object_mesh(&*self.resource, "dance_cam");
            let cb = c.borrow();
            camo.position = cb.object.read().unwrap().world_position();
            camo.orientation = transform::Orientation::new_with_quat(&cb.object.read().unwrap().world_orientation());
            camo.scale = cb.object.read().unwrap().world_scale();
            cams.push(Arc::new(RwLock::new(camo)));
        }
        if cams.is_empty() {
            panic!("cam is empty");
        }

        let not_loaded = self.render.draw(
            obs,
            &cams,
            sel,
            &self.dragger.borrow().get_objects(),
            &finish,
            self.loading_resource.clone());

        self.updating.set(false);

        not_loaded > 0
    }

    fn resize(&mut self, w : c_int, h : c_int)
    {
        self.width = w;
        self.height = h;
        self.render.resize(w, h);
    }
    
    fn handle_control_change(&self, change : &operation::Change)
    {
    }

    pub fn handle_event(&self, event : &ui::EventOld)
    {
        match *event {
            ui::Event::RectVisibleSet(b) => {
                if let Some(w) = self.window {
                    unsafe {
                        window_rect_visible_set(w, b);
                    }
                }
            },
            ui::Event::RectSet(x,y,w,h) => {
                if let Some(win) = self.window {
                    unsafe {
                        window_rect_set(win, x,y,w,h);
                    }
                }
            },
            _ => {}
        }
    }

    pub fn get_camera_transform(&self) -> (vec::Vec3, vec::Quat)
    {
        let c = self.camera.borrow();
        let c = c.object.read().unwrap();
        (c.position, c.orientation.as_quat())
    }

    pub fn request_update(&self)
    {
        if self.updating.get() {
            return;
        }

        if let Some(w) = self.window {
            self.updating.set(true);
            unsafe {ui::jk_window_request_update(w);}
        }
    }


}

/*
pub struct WindowView
{
    pub window : Option<*const Window>,
    pub view : View
}
*/

pub extern fn mouse_down(
    data : *const c_void,
    modifier : c_int,
    button : c_int,
    x : c_int,
    y : c_int,
    timestamp : c_int
    )
{
    let wcb : & ui::WidgetCbData = unsafe {&* (data as *const ui::WidgetCbData)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();

    let op_list = {
        let control_rc = container.views[wcb.index].control.clone();

        //println!("rust mouse down button {}, pos: {}, {}", button, x, y);
        let mut c = control_rc.borrow_mut();
        c.mouse_down(&*container.state.context, modifier, button,x,y,timestamp)
    };

    for op in &op_list {
        if let operation::Change::DraggerClicked = *op {
            //let c = &mut container.context;;
            container.state.save_positions();
            container.state.save_scales();
            container.state.save_oris();
        }
        container.views[wcb.index].handle_control_change(op);
        let id = container.views[wcb.index].uuid;
        container.handle_change(op, id);
    }
}

pub extern fn mouse_up(
    data : *const c_void,
    modifier : c_int,
    button : c_int,
    x : c_int,
    y : c_int,
    timestamp : c_int
    )
{
    let wcb : & ui::WidgetCbData = unsafe {&* (data as *const ui::WidgetCbData)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();

    let event = {
        let control_rc = container.views[wcb.index].control.clone();
        let mut c = control_rc.borrow_mut();
        c.mouse_up(&*container.state.context,button,x,y,timestamp)
    };

    container.views[wcb.index].handle_event(&event);
    let id = container.views[wcb.index].uuid;
    //TODO
    //container.handle_change(&change, id);
}

pub extern fn mouse_move(
    data : *const c_void,
    modifiers_flag : c_int,
    button : c_int,
    curx : c_int,
    cury : c_int,
    prevx : c_int,
    prevy : c_int,
    timestamp : c_int
    )
{
    let wcb : & ui::WidgetCbData = unsafe {&* (data as *const ui::WidgetCbData)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();
    let control_rc = container.views[wcb.index].control.clone();

    let change_list = {
        let mut c = control_rc.borrow_mut();
        c.mouse_move(
            &*container.state.context,
            modifiers_flag,
            button,
            curx,
            cury,
            prevx,
            prevy,
            timestamp)
    };

    let id = container.views[wcb.index].uuid;
    for change in &change_list {
        container.views[wcb.index].handle_control_change(change);
        container.handle_change(change, id);
    }
}

pub extern fn mouse_wheel(
    data : *const c_void,
    modifiers_flag: c_int,
    direction : c_int,
    z : c_int,
    x : c_int,
    y : c_int,
    timestamp : c_int
    )
{
    let wcb : & ui::WidgetCbData = unsafe {&* (data as *const ui::WidgetCbData)};
    let view : &View = &*wcb.container.read().unwrap().views[wcb.index];
    let control_rc = view.control.clone();

    let c = control_rc.borrow_mut();
    c.mouse_wheel(modifiers_flag, direction, z, x, y, timestamp);

    view.request_update();
}

pub extern fn key_down(
    data : *const c_void,
    modifier : c_int,
    keyname : *const c_char,
    key : *const c_char,
    keycode : c_uint,
    timestamp : c_int
    )
{
    let wcb : & ui::WidgetCbData = unsafe {&* (data as *const ui::WidgetCbData)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();

    let change = {
        let key_str = {
            let s = unsafe {CStr::from_ptr(key).to_bytes()};
            match str::from_utf8(s) {
                Ok(ss) => ss.to_owned(),
                _ => {
                    println!("error");
                    return;
                }
            }
        };

        let keyname_str = {
            let keynameconst = keyname as *const c_char;
            let s = unsafe {CStr::from_ptr(keynameconst).to_bytes()};
            match str::from_utf8(s) {
                Ok(ss) => ss.to_owned(),
                _ => {
                    println!("error");
                    return
                }
            }
        };

        match key_str.as_ref() {
            "Return" => {
                if let Some(ref mut cmd) = container.command {
                    println!("pressed return show popup");

                    cmd.clean();

                    let scene_actions : &[(&str, extern fn(*const c_void, *const c_char))]
                    = &[
                    ("add empty", ui::command::add_empty),
                    ("remove selected22", ui::command::remove_selected2),
                    ("set camera2", ui::command::set_camera2),
                    ("add component", ui::command::add_component),
                    ("copy selected", ui::command::copy_selected),
                    ];

                    for a in scene_actions {
                        let (ref name, f) = *a;
                        cmd.add_ptr(name, f, data);
                    }

                    cmd.show();
                }
                return;
            },
            "t" => {
                if let Some(ref mut t) = container.tree {
                    let b = t.visible();
                    t.set_visible(!b);
                }
                else {
                    println!("container does not have a tree");
                }
                return;
            },
            "p" => {
                let p = &mut container.property;
                let b = p.visible();
                p.set_visible(!b);
                return;
            },
            "a" => {
                if let Some(ref mut a) = container.action {
                    let b = a.visible();
                    a.set_visible(!b);
                }
                return;
            },
            "c" => {
                let center = vec::Vec3::zero();
                let mut cam = container.views[wcb.index].camera.borrow_mut();
                let pos = center + cam.object.read().unwrap().orientation.rotate_vec3(&vec::Vec3::new(0f64,0f64,100f64));
                cam.set_position(pos);
                cam.set_center(&center);
                container.views[wcb.index].request_update();
                return;
            },
            "f" => {
                let center = util::objects_center(&container.state.context.selected);
                let mut cam = container.views[wcb.index].camera.borrow_mut();
                let pos = center + cam.object.read().unwrap().orientation.rotate_vec3(&vec::Vec3::new(0f64,0f64,100f64));
                cam.set_position(pos);
                container.views[wcb.index].request_update();
                return;
            },
            _ => {
                println!("key not implemented : {}", key_str);
            }
        }

        {
            let control_rc = container.views[wcb.index].control.clone();
            let mut c = control_rc.borrow_mut();
            c.key_down(modifier, keyname_str.as_ref(), key_str.as_ref(), timestamp)
        }
    };

    container.views[wcb.index].handle_control_change(&change);
    let id = container.views[wcb.index].uuid;
    container.handle_change(&change, id);
}


/*
//TODO remove
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
*/


pub extern fn init_cb(data : *const c_void) -> () {
    let wcb : & ui::WidgetCbData = unsafe {&* (data as *const ui::WidgetCbData)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();
    let view = &mut container.views[wcb.index];

    return view.init_render();
}

pub extern fn request_update_again(data : *const c_void) -> bool
{
    let wcb : &ui::WidgetCbData = unsafe {&* (data as *const ui::WidgetCbData)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();
    let view = &mut container.views[wcb.index];

    if let Ok(lr) = view.loading_resource.try_lock() {
        if *lr == 0 {
            view.request_update();
            return false;
        }
    }
    true
}


pub extern fn draw_cb(data : *const c_void) -> ()
{
    let wcb : &ui::WidgetCbData = unsafe {&* (data as *const ui::WidgetCbData)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();

    let draw_not_done = container.views[wcb.index].draw(&*container.state.context);

    if draw_not_done {
        unsafe {
            ui::ecore_animator_add(request_update_again, data);
        }
    }
}

pub extern fn resize_cb(data : *const c_void, w : c_int, h : c_int) -> () {
    let wcb : & ui::WidgetCbData = unsafe {&* (data as *const ui::WidgetCbData)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();
    let view = &mut container.views[wcb.index];

    return view.resize(w, h);
}

fn create_camera_object_mesh(
    resource : &resource::ResourceGroup,
    name : &str) -> object::Object
{
   // let mut cam = factory.create_object(name);
        let mut cam = object::Object {
            name : String::from(name),
            id : uuid::Uuid::new_v4(),
            mesh_render : None,
            position : vec::Vec3::zero(),
            //orientation : vec::Quat::identity(),
            orientation : transform::Orientation::new_quat(),
            //angles : vec::Vec3::zero(),
            scale : vec::Vec3::one(),
            children : Vec::new(),
            parent : None,
            //transform : box transform::Transform::new()
            components : Vec::new(),
            comp_data : Vec::new(),
            comp_string : Vec::new(),
            comp_lua : Vec::new(),
        };


    let mat = create_mat();

    cam.mesh_render = Some(mesh_render::MeshRenderer::new_with_mat(
        "model/camera.mesh",
        mat,
        resource));

    cam
}

fn create_mat() -> material::Material
{
    let mut mat = material::Material::new_from_file("material/camera.mat");

    mat.set_uniform_data(
        "color",
        shader::UniformData::Vec4(vec::Vec4::new(1.1f64,0f64,0.1f64,0.2f64)));

    mat
}

