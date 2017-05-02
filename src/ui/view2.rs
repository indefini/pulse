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
use dormin::input;
use dormin::scene;
use dormin::render;

pub trait DataT<S : SceneT> {
    fn get_scene(&self, id : S::Id) -> Option<&S>;
}

pub trait SceneT {
    type Id : Default;
    fn update(&mut self, dt : f64, input : &input::Input, &resource::ResourceGroup);
}

pub trait RenderT<S> {
    //type Object;
    //fn draw(objs : &[Self::Object], loading : Arc<Mutex<usize>);
}

/*
impl RenderT<Rc<RefCell<scene::Scene>>> for GameRender
{

}
*/


trait ViewT<Scene:SceneT> {
    fn draw(&mut self, scene : &Scene) -> bool;
    fn init(&mut self);
    fn resize(&mut self, w : c_int, h : c_int);
    fn get_scene_id(&self) -> Scene::Id;
}

impl ViewT<Rc<RefCell<scene::Scene>>> for View2<render::GameRender,Rc<RefCell<scene::Scene>>> {

    fn draw(&mut self, scene : &Rc<RefCell<scene::Scene>>) -> bool
    {
        //TODO
        self.render.draw(&scene.borrow().objects, self.loading_resource.clone());
        false
    }

    fn init(&mut self) {
        self.render.init();
    }

    fn resize(&mut self, w : c_int, h : c_int)
    {
        self.render.resize(w, h);
    }

    fn get_scene_id(&self) -> uuid::Uuid
    {
        self.scene_id
    }

}

pub struct View2<R, S : SceneT>
{
    window : *const ui::Evas_Object,
    glview : *const ui::JkGlview,

    scene_id : S::Id,
    render : R,

    pub state : i32,
    pub loading_resource : Arc<Mutex<usize>>,
}

impl<R, S:SceneT> View2<R,S> {
    pub fn new(
        win : *const ui::Evas_Object,
        //dispatcher : Rc<Dispatcher<S>>,
        dispatcher : Rc<DispaTest>,
        //r : R ) -> Box<Box<View2<R,S>>> where Dispatcher<S> : DataT<S>
        r : R ) -> Box<Box<View2<R,S>>> where DispaTest : DataT<S>
    {
        //let render = box GameRender::new(camera, resource.clone());

        let mut v = box box View2 {
            window : win,
            //name : "cacayop".to_owned(),
            state : 0,
            scene_id : Default::default(),
            render : r,
            glview : ptr::null(),
            loading_resource : Arc::new(Mutex::new(0)),
            //camera : camera todo
        };

        let gldata = GlViewData::new(dispatcher, &mut *v as *mut Box<View2<R,S>> as *mut Box<ViewT<S>>);

        v.glview = unsafe { ui::jk_glview_new(
                win,
                //mem::transmute(&*v),
                mem::transmute(box gldata),
                gv_init_cb::<S>,
                gv_draw_cb::<S>,
                gv_resize_cb::<S>,
                gv_key_down::<S>,
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

pub extern fn gv_init_cb<S:SceneT>(v : *const c_void) {
    unsafe {
        //let gv : *mut View2 = mem::transmute(v);
        //let gv : &mut Box<ViewT<S>> = mem::transmute(gldata.view);
        //println!("AAAAAAAAAAAAAAAAAAAAAA gv init cb {}", (*gv).name);
        let gldata : &Box<GlViewData<S>> = mem::transmute(v);
        (*gldata.view).init();
    }
}

extern fn request_update_again_view2<Scene>(data : *const c_void) -> bool
{
    //let gv : &mut View2 =  unsafe {mem::transmute(data)};
    let gv : &mut Box<ViewT<Scene>> =  unsafe {mem::transmute(data)};

    //TODO
    /*
    //if let Ok(lr) = (*gv).loading_resource.try_lock() {
    if let Ok(lr) = gv.loading_resource.try_lock() {
        if *lr == 0 {
            //(*gv).request_update();
            gv.request_update();
            return false;
        }
    }
    */
    true
}


//pub extern fn gv_draw_cb<S:SceneT>(v : *const c_void) where Dispatcher<S> : DataT<S>
pub extern fn gv_draw_cb<S:SceneT>(v : *const c_void) where DispaTest : DataT<S>
{
    unsafe {
        //let gv : *mut View2 = mem::transmute(v);
        //let gv : *mut Box<ViewT<Scene>> = mem::transmute(v);
        //println!("draw {}", (*gv).name);
        //TODO TODO //TODO
        //let draw_not_done = (*gv).draw();
        
        let gldata : &Box<GlViewData<S>> = mem::transmute(v);
        let scene = gldata.dis.get_scene((*gldata.view).get_scene_id());
        (*gldata.view).draw(scene.unwrap());

        //TODO
        /*
        if draw_not_done && (*gv).state == 0 {
                ui::ecore_animator_add(request_update_again_view2, mem::transmute(v));
        }
        */
    }
}

pub extern fn gv_resize_cb<S:SceneT>(v : *const c_void, w : c_int, h : c_int) {
    unsafe {
        //return (*v).resize(w, h);
        //let gv : *mut View2 = mem::transmute(v);
        let gv : *mut Box<ViewT<S>> = mem::transmute(v);
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

extern fn gv_key_down<S>(
    data : *const c_void,
    modifier : c_int,
    keyname : *const c_char,
    key : *const c_char,
    keycode : c_uint,
    timestamp : c_int)
{
    //let gv : *mut View2 = unsafe { mem::transmute(data) };
    //let gv : &mut View2 = unsafe { &mut *gv };
    let gv : *mut Box<ViewT<S>> = unsafe { mem::transmute(data) };
    let gv : &mut Box<ViewT<S>> = unsafe { &mut *gv };
    //unsafe { (*gv).input.add_key(keycode as u8); }
    println!("key pressed {}", (keycode as u8));
}

trait SceneUpdate {
    fn update(&mut self, dt : f64)
    {
            //self.scene.borrow_mut().update(0.01f64, &self.input, &*self.resource);
    }
}

struct SceneS;
impl SceneT for SceneS {
    type Id = usize;
    fn update(&mut self, dt : f64, input : &input::Input, res :&resource::ResourceGroup)
    {

    }
}

use uuid;
impl SceneT for Rc<RefCell<scene::Scene>> {
    type Id = uuid::Uuid;
    fn update(&mut self, dt : f64, input : &input::Input, res :&resource::ResourceGroup)
    {

    }
}

pub struct DispaTest;



pub struct Dispatcher<S:SceneT>
{
    scenes : Vec<S>
}

impl<S:SceneT> Dispatcher<S> {
    fn new() -> Dispatcher<S> {
        Dispatcher {
            scenes : Vec::new()
        }
    }
}

impl DataT<SceneS> for Dispatcher<SceneS> {
    fn get_scene(&self, id : usize) -> Option<&SceneS>
    {
        None
    }
}

/*
impl<S:SceneT> DataT<S> for Dispatcher<S> {
    fn get_scene(&self, id : S::Id) -> Option<&S>
    {
        None
    }
}
*/


impl DataT<Rc<RefCell<scene::Scene>>> for Dispatcher<Rc<RefCell<scene::Scene>>> {
    fn get_scene(&self, id : uuid::Uuid) -> Option<&Rc<RefCell<scene::Scene>>>
    {
        None
    }
}

impl DataT<Rc<RefCell<scene::Scene>>> for DispaTest {
    fn get_scene(&self, id : uuid::Uuid) -> Option<&Rc<RefCell<scene::Scene>>>
    {
        None
    }
}


struct GlViewData<Scene:SceneT> {
    //dis : Rc<Dispatcher<Scene>>,
    dis : Rc<DispaTest>,
    view : *mut Box<ViewT<Scene>>,
}

impl<S:SceneT> GlViewData<S> {
    //fn new(d : Rc<Dispatcher<S>>, view : *mut Box<ViewT<S>>) -> GlViewData<S>
    fn new(d : Rc<DispaTest>, view : *mut Box<ViewT<S>>) -> GlViewData<S>
    {
        GlViewData {
            dis : d,
            view : view

        }
    }
}
