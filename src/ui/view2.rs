use std::rc::Rc;
use std::cell::{RefCell};
use std::sync::{Arc,Mutex};
use libc::{c_char, c_void, c_int, c_uint};
use std::mem;
use std::ptr;

use ui;
use dormin::resource;
use dormin::camera;
use util::Arw;

pub struct View2
{
    window : *const ui::Evas_Object,
    glview : *const ui::JkGlview,

    //name : String,
    pub state : i32,
    pub loading_resource : Arc<Mutex<usize>>,
}

impl View2 {
    pub fn new(win : *const ui::Evas_Object) -> Box<View2>
    {
        //let render = box GameRender::new(camera, resource.clone());

        let mut v = box View2 {
            window : win,
            //name : "cacayop".to_owned(),
            state : 0,
            glview : ptr::null(),
            loading_resource : Arc::new(Mutex::new(0)),
            //camera : camera todo
        };


        v.glview = unsafe { ui::jk_glview_new(
                win,
                mem::transmute(&*v),
                gv_init_cb,
                gv_draw_cb,
                gv_resize_cb,
                gv_key_down,
                ) };

        v.set_visible(true);

        return v;
    }

    pub fn update(&mut self) -> bool {
        if self.state == 1 {
            //TODO scene update
            //self.scene.borrow_mut().update(0.01f64, &self.input, &*self.resource);
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
        //TODO
        //self.render.draw(&self.scene.borrow().objects, self.loading_resource.clone())
        false
    }

    fn init(&mut self) {
        //TODO
        //self.render.init();
    }

    fn resize(&mut self, w : c_int, h : c_int)
    {
        //TODO
        //self.render.resize(w, h);
    }

    pub fn set_visible(&mut self, b : bool)
    {
        if b {
            unsafe { ui::evas_object_show(self.window); }
        }
        else {
            unsafe { ui::evas_object_hide(self.window); }
        }
    }
}

pub extern fn gv_init_cb(v : *const c_void) {
    unsafe {
        let gv : *mut View2 = mem::transmute(v);
        //println!("AAAAAAAAAAAAAAAAAAAAAA gv init cb {}", (*gv).name);
        (*gv).init();
    }
}

extern fn request_update_again_view2(data : *const c_void) -> bool
{
    //let gv : *mut View2 =  unsafe {mem::transmute(data)};
    let gv : &mut View2 =  unsafe {mem::transmute(data)};

    //if let Ok(lr) = (*gv).loading_resource.try_lock() {
    if let Ok(lr) = gv.loading_resource.try_lock() {
        if *lr == 0 {
            //(*gv).request_update();
            gv.request_update();
            return false;
        }
    }
    true
}


pub extern fn gv_draw_cb(v : *const c_void) {
    unsafe {
        let gv : *mut View2 = mem::transmute(v);
        //println!("draw {}", (*gv).name);
        let draw_not_done = (*gv).draw();

        if draw_not_done && (*gv).state == 0 {
            unsafe {
                ui::ecore_animator_add(request_update_again_view2, mem::transmute(v));
            }
    }
    }
}

pub extern fn gv_resize_cb(v : *const c_void, w : c_int, h : c_int) {
    unsafe {
        //return (*v).resize(w, h);
        let gv : *mut View2 = mem::transmute(v);
        //println!("resize {}", (*gv).name);
        (*gv).resize(w, h);
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
    let gv : *mut View2 = unsafe { mem::transmute(data) };
    let gv : &mut View2 = unsafe { &mut *gv };
    //unsafe { (*gv).input.add_key(keycode as u8); }
    println!("key pressed {}", (keycode as u8));
}

