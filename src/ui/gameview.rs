use std::rc::Rc;
use std::cell::{RefCell};
use std::sync::{Arc,Mutex};
use libc::{c_char, c_void, c_int, c_uint};
use std::mem;
use std::ptr;
use uuid;

use ui;
use dormin::render;
use render::{GameRender};
use dormin::input;
use ui::def::Widget;
use data::{Data,SceneT};


pub trait GameViewTrait<S:SceneT> : ui::Widget<S> {
    fn play(&mut self);
    fn pause(&mut self);
    fn stop(&mut self);
    fn get_scene_id(&self) -> S::Id;

    //TODO check if this belong here.
    fn update(&mut self) -> bool;
    fn get_input(&self) -> &input::Input;
    fn request_update(&self);
}

impl<S:SceneT> ui::Widget<S> for GameView<S> {
 
    fn set_visible(&mut self, b : bool)
    {
        self.config.visible = b;

        if b {
            unsafe { ui::evas_object_show(self.window); }
        }
        else {
            unsafe { ui::evas_object_hide(self.window); }
        }
    }

    fn get_id(&self) -> uuid::Uuid
    {
        println!("TODO uuid");
        uuid::Uuid::nil()
    }

    fn get_config(&self) -> ui::WidgetConfig
    {
        self.config.clone()
    }
}

pub struct GameView<S:SceneT>
{
    window : *const ui::Evas_Object,
    pub glview : *const ui::JkGlview,
    render : Box<GameRender<S>>,
    pub scene : uuid::Uuid,
    name : String,
    pub state : i32,
    pub input : input::Input,
    pub config : ui::WidgetConfig,
    pub loading_resource : Arc<Mutex<usize>>,
    //data : *const Box<Data<Scene>>,
    data : *const c_void
}



impl<S:SceneT> GameView<S> {
    pub fn new(
        win : *const ui::Evas_Object,
        scene : uuid::Uuid,
        //data : *const Box<Data<Scene>>,
        data : *const c_void,
        render : Box<GameRender<S>>,
        config : ui::WidgetConfig
        ) -> Box<GameView<S>>
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
            data : data
            //resource : resource
            //camera : camera todo
        };

        v.glview = unsafe { ui::jk_glview_new(
                win,
                //mem::transmute(&*v.render),
                mem::transmute(&*v),
                gv_init_cb::<S>,
                gv_draw_cb::<S>,
                gv_resize_cb::<S>,
                gv_key_down::<S>,
                ) };

        println!("TODO uncomment not visible because of this {}, {}", file!(), line!());
        //v.set_visible(config.visible);

        return v;
    }
 
    fn draw(&mut self) -> bool
    {
        println!("TODO remove this? {}, {}", file!(), line!());
        false
        /*
        let id = self.get_scene_id();
        //let s = unsafe { (&*self.data).get_scene(id) };
        //let data : & Box<DataT<Scene>> = self.data as &Box<DataT<Scene>>;
        //let s = (*self.data).get_scene(id);
        let s = unsafe { (*self.data).get_scene(id) };

        if let Some(scene) = s
        {
            let cam_id_mat = if let Some(ref camera) = scene.borrow().camera {
                let mut camera = camera.borrow_mut();
                camera.set_resolution(self.config.w,self.config.h);
                camera.to_cam_id_mat()
            }
            else {
                return false;
            };

            self.render.draw(
                &cam_id_mat,
                &scene.get_mmr(),
                self.loading_resource.clone())

        }
        else {
            false
        }
        //self.render.draw(&self.scene.borrow().objects, self.loading_resource.clone())
        */
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

    pub fn clear_input(&mut self)
    {
        self.input.clear();
    }
}

pub extern fn gv_init_cb<S:SceneT>(v : *const c_void) {
    let gv : &mut GameView<S> = unsafe { mem::transmute(v) };
    //println!("AAAAAAAAAAAAAAAAAAAAAA gv init cb {}", (*gv).name);
    gv.init();
}

pub extern fn request_update_again_gv<S:SceneT>(data : *const c_void) -> bool
{
    let gv : &mut GameView<S> =  unsafe {mem::transmute(data)};

    if let Ok(lr) = gv.loading_resource.try_lock() {
        if *lr == 0 {
            println!("TODO commented for compile, {}, {}", file!(), line!());
            //gv.request_update();
            return false;
        }
    }
    true
}


pub extern fn gv_draw_cb<S:SceneT>(v : *const c_void)
{
    let gv : &mut GameView<S> = unsafe { mem::transmute(v) };
    //println!("draw {}", (*gv).name);
    let draw_not_done = gv.draw();

    if draw_not_done && gv.state == 0 {
        unsafe {
            ui::ecore_animator_add(request_update_again_gv::<S>, mem::transmute(v));
        }
    }
}

pub extern fn gv_resize_cb<S:SceneT>(v : *const c_void, w : c_int, h : c_int) {
    unsafe {
        let gv : &mut GameView<S> = mem::transmute(v);
        //println!("resize {}", (*gv).name);
        gv.resize(w, h);
    }
}

extern fn gv_key_down<S:SceneT>(
    data : *const c_void,
    modifier : c_int,
    keyname : *const c_char,
    key : *const c_char,
    keycode : c_uint,
    timestamp : c_int)
{
    let gv : &mut GameView<S> = unsafe { mem::transmute(data) };
    gv.input.add_key(keycode as u8);
}

