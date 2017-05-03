use std::rc::Rc;
use std::cell::{Cell,RefCell, BorrowState};
use std::sync::{RwLock, Arc,Mutex};
use libc::{c_char, c_void, c_int, c_uint};
use std::mem;
use std::ptr;

use ui;
use dormin::render::{Render, GameRender};
use dormin::resource;
use dormin::camera;
use dormin::scene;
use util::Arw;
use dormin::input;



pub struct GameView
{
    window : *const ui::Evas_Object,
    glview : *const ui::JkGlview,
    render : Box<GameRender>,
    pub scene : Rc<RefCell<scene::Scene>>,
    name : String,
    pub state : i32,
    input : input::Input,
    pub config : ui::WidgetConfig,
    pub loading_resource : Arc<Mutex<usize>>,
    //resource : Rc<resource::ResourceGroup>,
}



impl GameView {
    pub fn new(
        //factory: &mut factory::Factory,
        win : *const ui::Evas_Object,
        scene : Rc<RefCell<scene::Scene>>,
        render : Box<GameRender>,
        config : ui::WidgetConfig
        ) -> Box<GameView>
    {
        /*
        let camera = Rc::new(RefCell::new(factory.create_camera()));
        {
            let mut cam = camera.borrow_mut();
            cam.pan(&vec::Vec3::new(100f64,20f64,100f64));
            cam.lookat(vec::Vec3::new(0f64,5f64,0f64));
        }
        */

        /*
        let win = unsafe {
            ui::jk_window_new(gv_close_cb, ptr::null())
        };
        */

        let mut v = box GameView {
            render : render,
            window : win,
            scene : scene,
            name : "cacayop".to_owned(),
            state : 0,
            glview : ptr::null(),
            input : input::Input::new(),
            config : config.clone(),
            loading_resource : Arc::new(Mutex::new(0)),
            //resource : resource
            //camera : camera todo
        };


        v.glview = unsafe { ui::jk_glview_new(
                win,
                //mem::transmute(&*v.render),
                mem::transmute(&*v),
                gv_init_cb,
                gv_draw_cb,
                gv_resize_cb,
                gv_key_down,
                ) };

        v.set_visible(config.visible);

        return v;
    }

    pub fn update(&mut self) -> bool {
        if self.state == 1 {
            unsafe { ui::jk_glview_request_update(self.glview); }
            true
        }
        else {
            //unsafe { jk_glview_request_update(self.glview); }
            false
        }
    }
    
    pub fn request_update(&self)
    {
        unsafe { ui::jk_glview_request_update(self.glview); }
    }

    fn draw(&mut self) -> bool
    {
        self.render.draw(&self.scene.borrow().objects, self.loading_resource.clone())
    }

    fn init(&mut self) {
        self.render.init();
    }

    fn resize(&mut self, w : c_int, h : c_int)
    {
        self.render.resize(w, h);
        self.config.w = w;
        self.config.h = h;
    }

    pub fn visible(&self) -> bool
    {
        self.config.visible
    }

    pub fn get_config(&self) -> ui::WidgetConfig
    {
        self.config.clone()
    }

    pub fn set_visible(&mut self, b : bool)
    {
        self.config.visible = b;

        if b {
            unsafe { ui::evas_object_show(self.window); }
        }
        else {
            unsafe { ui::evas_object_hide(self.window); }
        }
    }

    pub fn get_input(&self) -> &input::Input
    {
        &self.input
    }

    pub fn clear_input(&mut self)
    {
        self.input.clear();
    }
}

pub extern fn gv_init_cb(v : *const c_void) {
    let gv : &mut GameView = unsafe { mem::transmute(v) };
    //println!("AAAAAAAAAAAAAAAAAAAAAA gv init cb {}", (*gv).name);
    gv.init();
}

pub extern fn request_update_again_gv(data : *const c_void) -> bool
{
    let gv : &mut GameView =  unsafe {mem::transmute(data)};

    if let Ok(lr) = gv.loading_resource.try_lock() {
        if *lr == 0 {
            gv.request_update();
            return false;
        }
    }
    true
}


pub extern fn gv_draw_cb(v : *const c_void)
{
    let gv : &mut GameView = unsafe { mem::transmute(v) };
    //println!("draw {}", (*gv).name);
    let draw_not_done = gv.draw();

    if draw_not_done && gv.state == 0 {
        unsafe {
            ui::ecore_animator_add(request_update_again_gv, mem::transmute(v));
        }
    }
}

pub extern fn gv_resize_cb(v : *const c_void, w : c_int, h : c_int) {
    unsafe {
        let gv : &mut GameView = mem::transmute(v);
        //println!("resize {}", (*gv).name);
        gv.resize(w, h);
    }
}

pub extern fn gv_close_cb(data : *mut c_void) {
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(data)};
    let container : Box<Arw<ui::WidgetContainer>> = unsafe {mem::transmute(data)};
    let container = &mut *container.write().unwrap();
    if let Some(ref mut gv) = container.gameview {
        gv.set_visible(false);
    }
}

extern fn gv_key_down(
    data : *const c_void,
    modifier : c_int,
    keyname : *const c_char,
    key : *const c_char,
    keycode : c_uint,
    timestamp : c_int)
{
    let gv : &mut GameView = unsafe { mem::transmute(data) };
    gv.input.add_key(keycode as u8);
}

