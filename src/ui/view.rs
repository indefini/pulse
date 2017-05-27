use std::rc::Rc;
use std::cell::{Cell};
use std::sync::{Arc,Mutex};
use libc::{c_char, c_void, c_int, c_uint, c_float};
use std::ffi::CStr;
use std::str;
use uuid;
use dormin::shader;
use dormin::transform;

use ui;
use dormin::render;
use dormin::render::Render;
use dormin::resource;
use dormin::vec;
use dormin::material;
use dormin::{camera2};
use control::Control;
use dormin::component::mesh_render;
use util;
use context;
use data::SceneT;


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

//TODO check if all functions belong here
pub trait EditView<S:SceneT> : ui::Widget {
    //fn play(&mut self);
    //fn get_scene_id(&self) -> Option<S::Id>;

    fn init(
        &mut self,
        win : *const ui::Window,
        wcb : ui::WidgetCbData
        );

    //TODO clean camera functions
    fn get_camera(&self) -> &CameraView;

    fn request_update(&self);

    //TODO user input
    fn handle_event(&self, event : &ui::EventOld);


    //TODO glview cb and draw stuff
    fn init_render(&mut self);
    fn draw(&mut self, context : &context::ContextOld) -> bool;
    fn resize(&mut self, w : c_int, h : c_int);
    fn is_loading_resource(&self) -> bool;

    fn mouse_down(
            &mut self,
            context : &context::ContextOld,
            modifier : i32,
            button : i32,
            x : i32,
            y : i32,
            timestamp : i32) -> Vec<ui::EventOld>;

    fn mouse_up(
            &mut self,
            context : &context::ContextOld,
            button : i32,
            x : i32,
            y : i32,
            timestamp : i32) -> ui::EventOld;

    fn mouse_move(
        &mut self,
        context : &context::ContextOld,
        mod_flag : i32,
        button : i32,
        curx : i32,
        cury : i32,
        prevx : i32,
        prevy : i32,
        timestamp : i32) -> Vec<ui::EventOld>;

    fn mouse_wheel(
        &mut self,
        modifier : i32,
        direction : i32,
        z : i32,
        x : i32,
        y : i32,
        timestamp : i32
        );

    fn key_down(
        &mut self,
        context : &context::ContextOld,
        modifier : i32,
        keyname : &str,
        key : &str,
        timestamp : i32
        ) ->  Vec<ui::EventOld>;

}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CameraView
{
    pub property : camera2::Camera,
    pub transform : transform::Transform,

    pub yaw : f64,
    pub pitch : f64,
    pub roll : f64,

    pub origin : vec::Vec3,
    pub local_offset : vec::Vec3,
    pub center : vec::Vec3,
}

impl CameraView
{
    pub fn to_camera2_transform(&self) -> camera2::CameraTransform
    {
        camera2::CameraTransform::new(&self.transform, &self.property)
    }

    pub fn pan(&mut self, t : &vec::Vec3)
    {
        let o = &mut self.transform;

        self.local_offset = self.local_offset + *t;
        let tt = o.orientation.rotate_vec3(t);
        o.position = o.position + tt;
    }

    pub fn set_center(&mut self, c : &vec::Vec3)
    {
        self.center = *c;
        self.recalculate_origin();
    }

    fn recalculate_origin(&mut self)
    {
        let offset = self.transform.orientation.rotate_vec3(&self.local_offset);
        let origin = self.transform.position - offset;
        let qi = self.transform.orientation.as_quat().inverse();
        self.origin = qi.rotate_vec3_around(&origin, &self.center);
    }

    pub fn rotate_around_center(&mut self, q : &vec::Quat)
    {
        let def = q.rotate_vec3_around(&self.origin, &self.center);
        let doff = q.rotate_vec3(&self.local_offset);
        self.transform.position = def + doff;
    }

}


pub struct View
{
    render : Box<Render>,
    control : Control,
    camera : CameraView,

    window : Option<*const ui::Window>,

    uuid : uuid::Uuid,
    config : ui::WidgetConfig,

    updating : Cell<bool>,
    loading_resource : Arc<Mutex<usize>>,
}

impl<S:SceneT> EditView<S> for View
{
    /*
    fn get_scene_id(&self) -> Option<S::Id>
    {
        None
    }
    */

    fn init(
        &mut self,
        win : *const ui::Window,
        wcb : ui::WidgetCbData
        ) {

        self.init_(win, wcb);
    }

    fn get_camera(&self) -> &CameraView
    {
       &self.camera
    }

    fn request_update(&self)
    {
        if self.updating.get() {
            return;
        }

        if let Some(w) = self.window {
            self.updating.set(true);
            unsafe {ui::jk_window_request_update(w);}
        }
    }

    fn handle_event(&self, event : &ui::EventOld)
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

    fn init_render(&mut self)
    {
        self.render.init();
    }

    fn is_loading_resource(&self) -> bool
    {
        if let Ok(lr) = self.loading_resource.try_lock() {
            if *lr == 0 {
                return false;
            }
        }

        true
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
            let mut dragger = &mut self.control.dragger;
            dragger.set_position(center);
            dragger.set_orientation(transform::Orientation::Quat(ori), &self.camera.transform.position);
            //let scale = self.camera.borrow().get_camera_resize_w(0.05f64);
            //dragger.set_scale(scale);
            self.camera.transform.set_as_dirty();
            let cam_mat = self.camera.transform.get_or_compute_local_matrix();
            let projection = self.camera.property.get_perspective();
            let cam_mat_inv = cam_mat.get_inverse();

            dragger.scale_to_camera(&cam_mat_inv, &projection);
        }

        let finish = |b| {
            //println!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~render finished");
        };

        let mut cams = Vec::new();
        for c in &scene.cameras {
            let cb = c.borrow();
            let mat = cb.object.read().unwrap().get_world_matrix();
            let mr = mesh_render::MeshRender::new_with_mat2(
            "model/camera.mesh", create_mat());

            let mmr = render::MatrixMeshRender::new(mat, mr);
            cams.push(mmr);
        }
        if cams.is_empty() {
            panic!("cam is empty");
        }

        self.camera.transform.set_as_dirty();
        self.camera.transform.compute_local_matrix();
        let not_loaded = self.render.draw(
            &render::CameraIdMat::from_transform_camera2(
                uuid::Uuid::new_v4(),
                &self.camera.transform,
                &self.camera.property),
            obs,
            &cams,
            sel,
            &self.control.dragger.get_mmr(),
            &finish,
            self.loading_resource.clone());

        self.updating.set(false);

        not_loaded > 0
    }

    fn resize(&mut self, w : c_int, h : c_int)
    {
        self.config.w = w;
        self.config.h = h;
        self.camera.property.set_resolution(w, h);
        self.render.resize(w, h);
    }

    fn mouse_down(
            &mut self,
            context : &context::ContextOld,
            modifier : i32,
            button : i32,
            x : i32,
            y : i32,
            timestamp : i32) -> Vec<ui::EventOld>
    {
        self.control.mouse_down(&self.camera.to_camera2_transform(), context,  modifier, button, x, y, timestamp)
    }

    fn mouse_up(
            &mut self,
            context : &context::ContextOld,
            button : i32,
            x : i32,
            y : i32,
            timestamp : i32) -> ui::EventOld
    {
        self.control.mouse_up(&self.camera.to_camera2_transform(), context, button, x, y, timestamp)
    }

    fn mouse_move(
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
        self.control.mouse_move(&mut self.camera, context, mod_flag, button, curx, cury, prevx, prevy, timestamp)
    }

    fn mouse_wheel(
        &mut self,
        modifier : i32,
        direction : i32,
        z : i32,
        x : i32,
        y : i32,
        timestamp : i32
        )
    {
        self.control.mouse_wheel(&mut self.camera, modifier, direction, z, x, y, timestamp);
    }

    fn key_down(
        &mut self,
        context : &context::ContextOld,
        modifier : i32,
        keyname : &str,
        key : &str,
        timestamp : i32
        ) ->  Vec<ui::EventOld>
    {
        match key {
            "c" => {
                let ori = self.camera.transform.orientation;
                let center = vec::Vec3::zero();
                let pos = center + ori.rotate_vec3(&vec::Vec3::new(0f64,0f64,100f64));
                self.camera.transform.position = pos;
                self.camera.set_center(&center);
                return vec![ui::Event::CameraChange];
            },
            "f" => {
                let center = util::objects_center(&context.selected);
                let ori = self.camera.transform.orientation;
                let pos = center + ori.rotate_vec3(&vec::Vec3::new(0f64,0f64,100f64));
                self.camera.transform.position = pos;
                return vec![ui::Event::CameraChange];
            },
            _ => {
                println!("key not implemented : {}", key);
            }
        }

        self.control.key_down(&mut self.camera, modifier, keyname, key, timestamp)
    }
}

impl ui::Widget for View {
    fn set_visible(&mut self, b : bool)
    {
        self.config.visible = b;

        println!("todo set visible for view");
        /*
        if let Some(w) = self.window {
            if b {
                unsafe { ui::evas_object_show(w); }
            }
            else {
                unsafe { ui::evas_object_hide(w); }
            }
        }
        */
    }

    fn get_id(&self) -> uuid::Uuid
    {
        self.uuid
    }

    fn get_config(&self) -> ui::WidgetConfig
    {
        self.config.clone()
    }

}

impl View
{
    pub fn new(
        resource : Rc<resource::ResourceGroup>,
        render : Box<Render>,
        w : i32,
        h : i32,
        camera : CameraView
        ) -> View
    {
        View {
            render : render,
            control : Control::new(resource.clone()),

            window : None,

            camera : camera,
            uuid : uuid::Uuid::new_v4(),
            config : ui::WidgetConfig::with_width_height(w, h),
            updating : Cell::new(false),
            loading_resource : Arc::new(Mutex::new(0)),
        }
    }

    pub fn init_(
        &mut self,
        win : *const ui::Window,
        wcb : ui::WidgetCbData
        ) {

        self.window = Some(win);

        unsafe {
            //TODO clean Box::into_raw data
            ui::window_callback_set(
                win,
                Box::into_raw(box wcb.clone()) as *const c_void,
                mouse_down,
                mouse_up,
                mouse_move,
                mouse_wheel,
                key_down
                );

            //TODO clean Box::into_raw data
            ui::tmp_func(
                win,
                Box::into_raw(box wcb) as *const c_void,
                init_cb,
                draw_cb,
                resize_cb);
        }
    } 
}

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

    let op_list =
        container.views[wcb.index].mouse_down(
            &*container.state.context,
            modifier,
            button,
            x,
            y,
            timestamp);

    for op in op_list.into_iter() {
        if let ui::Event::DraggerClicked = op {
            container.state.save_positions();
            container.state.save_scales();
            container.state.save_oris();
        }
        container.views[wcb.index].handle_event(&op);
        let id = container.views[wcb.index].get_id();
        container.handle_event(op, id);
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

    let event =
        container.views[wcb.index].mouse_up(&*container.state.context, button, x, y, timestamp);

    container.views[wcb.index].handle_event(&event);
    let id = container.views[wcb.index].get_id();
    container.handle_event(event, id);
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

    let events =
        container.views[wcb.index].mouse_move(
            &*container.state.context,
            modifiers_flag,
            button,
            curx,
            cury,
            prevx,
            prevy,
            timestamp);

    let id = container.views[wcb.index].get_id();
    for e in events {
        container.views[wcb.index].handle_event(&e);
        container.handle_event(e, id);
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
    let view : &mut EditView<_> = &mut *wcb.container.write().unwrap().views[wcb.index];
    view.mouse_wheel(modifiers_flag, direction, z, x, y, timestamp);

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

    let event = {
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
            _ => {}
        }

        {
            let v = &mut container.views[wcb.index];
            v.key_down(&container.state.context, modifier, keyname_str.as_ref(), key_str.as_ref(), timestamp)
        }
    };

    for e in event {
        let id = {
            let view = &container.views[wcb.index];
            view.handle_event(&e);
            view.get_id()
        };
        container.handle_event(e, id);
    }
}

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

    if !view.is_loading_resource() {
        view.request_update();
        return false;
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

fn create_mat() -> material::Material
{
    let mut mat = material::Material::new_from_file("material/camera.mat");

    mat.set_uniform_data(
        "color",
        shader::UniformData::Vec4(vec::Vec4::new(1.1f64,0f64,0.1f64,0.2f64)));

    mat
}

