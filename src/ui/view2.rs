use std::rc::Rc;
use std::cell::{RefCell};
use std::sync::{Arc,Mutex};
use libc::{c_char, c_void, c_int, c_uint};
use std::mem;
use std::ptr;

use uuid;

use ui;
use util::Arw;
use dormin::input;
use dormin::render;
use data::{Data, SceneT};

/*
pub trait RenderT<S> {
    //type Object;
    //fn draw(objs : &[Self::Object], loading : Arc<Mutex<usize>);
}

impl RenderT<Rc<RefCell<scene::Scene>>> for GameRender
{

}
*/

pub trait ViewT<Scene:SceneT> {
    fn draw(&mut self, scene : &Scene) -> bool;
    fn init(&mut self);
    fn resize(&mut self, w : c_int, h : c_int);
    fn get_scene_id(&self) -> Scene::Id;
    fn handle_key(&mut self, key : u8) ;
}

pub struct View2<R, S : SceneT>
{
    window : *const ui::Evas_Object,
    glview : *const ui::JkGlview,
    id : uuid::Uuid,

    pub scene_id : S::Id,
    pub render : R,

    pub config : ui::WidgetConfig,
    pub state : i32,
    pub loading_resource : Arc<Mutex<usize>>,
    pub input : input::Input,
}

impl<R:'static, S:SceneT+'static> View2<R,S> where View2<R,S> : ViewT<S> {
    pub fn new(
        win : *const ui::Evas_Object,
        d : *const Data<S>,
        config : ui::WidgetConfig,
        r : R,
        id : S::Id
        ) -> Box<View2<R,S>> //where Dispatcher : DataT<S>
    {
        //let render = box GameRender::new(camera, resource.clone());

        let mut v = box View2 {
            window : win,
            id : uuid::Uuid::new_v4(),
            //name : "cacayop".to_owned(),
            scene_id : id, //Default::default(),
            render : r,
            glview : ptr::null(),
            config : config,
            state : 0,
            loading_resource : Arc::new(Mutex::new(0)),
            input : input::Input::new(),
            //camera : camera todo
        };

        let gldata = box GlViewData::new(
            d,
            &mut *v as *mut View2<R,S> as *mut ViewT<S>,
            &mut *v as *mut View2<R,S> as *mut ui::gameview::GameViewTrait<S>);

        v.glview = unsafe { ui::jk_glview_new(
                win,
                Box::into_raw(gldata) as *const c_void,
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
        let gldata : &GlViewData<S> = &*(v as *const GlViewData<S>) ;
        (*gldata.view).init();
    }
}

extern fn request_update_again_view2<Scene:SceneT>(data : *const c_void) -> bool
{
    //TODO check if loading is over before trying to render
    /*
    //if let Ok(lr) = (*gv).loading_resource.try_lock() {
    if let Ok(lr) = gv.loading_resource.try_lock() {
        if *lr == 0 {
            //(*gv).request_update();
            gv.request_update();
            return false;
        }
    }
    //true
    */

    unsafe {
        let gldata : &GlViewData<Scene> = &*(data as *const GlViewData<Scene>) ;

        let id = (*gldata.view).get_scene_id();
        if let Some(scene) = (*gldata.dis).get_scene(id) {
            (*gldata.gameview).request_update();
        }
    }

    false
}


pub extern fn gv_draw_cb<S:SceneT>(v : *const c_void) //where Dispatcher : DataT<S>
{
    unsafe {
        
        let gldata : &GlViewData<S> = &*(v as *const GlViewData<S>) ;

        let id = (*gldata.view).get_scene_id();
        if let Some(scene) = (*gldata.dis).get_scene(id) {

            let draw_not_done = (*gldata.view).draw(scene);

            if draw_not_done {
                ui::ecore_animator_add(request_update_again_view2::<S>, v);
            }
        }
    }
}

pub extern fn gv_resize_cb<S:SceneT>(v : *const c_void, w : c_int, h : c_int) {
    unsafe {
        let gldata : &GlViewData<S> = &*(v as *const GlViewData<S>) ;
        (*gldata.view).resize(w, h);
    }
}

pub extern fn gv_close_cb<S:SceneT>(data : *mut c_void) {
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(data)};
    let container : Box<Arw<ui::WidgetContainer<S>>> = unsafe {mem::transmute(data)};
    let container = &mut *container.write().unwrap();
    if let Some(ref mut gv) = container.gameview {
        gv.set_visible(false);
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
    println!("key pressed {}", (keycode as u8));
    unsafe {
        let gldata : &GlViewData<S> = &*(data as *const GlViewData<S>) ;
        (*gldata.view).handle_key(keycode as u8);
    }
}

trait SceneUpdate {
    fn update(&mut self, dt : f64)
    {
            //self.scene.borrow_mut().update(0.01f64, &self.input, &*self.resource);
    }
}

/*
struct SceneS;
impl SceneT for SceneS {
    type Id = usize;
    type Object = usize;
    fn update(&mut self, dt : f64, input : &input::Input, res :&resource::ResourceGroup)
    {
    }

    fn init_for_play(&mut self, resource : &resource::ResourceGroup)
    {
    }

    fn get_objects(&self) -> &[Self::Object]
    {
        &[]
    }
}

use data;
impl data::ToId<usize> for SceneS {
    fn to_id(&self) -> usize
    {
        0usize
    }
}
*/

/*
pub struct Dispatcher
{
    scenes : Vec<Rc<RefCell<scene::Scene>>>
}

impl DataT<Rc<RefCell<scene::Scene>>> for Dispatcher {
    fn get_scene(&self, id : uuid::Uuid) -> Option<&Rc<RefCell<scene::Scene>>>
    {
        for i in 0..self.scenes.len() {
            if self.scenes[i].borrow().id == id {
                return Some(&self.scenes[i]);
            }
        }

        None
    }

    fn get_scene_mut(&mut self, id : uuid::Uuid) -> Option<&mut Rc<RefCell<scene::Scene>>>
    {
        for i in 0..self.scenes.len() {
            if self.scenes[i].borrow().id == id {
                return Some(&mut self.scenes[i]);
            }
        }

        None
    }
}
*/

struct GlViewData<Scene:SceneT> {
    dis : *const Data<Scene>,
    view : *mut ViewT<Scene>,
    gameview : *mut ui::gameview::GameViewTrait<Scene>,
}

impl<S:SceneT> GlViewData<S> {
    fn new(d : *const Data<S>, view : *mut ViewT<S>, gameview : *mut ui::gameview::GameViewTrait<S>) -> GlViewData<S>
    {
        GlViewData {
            dis : d,
            view : view,
            gameview : gameview
        }
    }
}

impl<R,S:SceneT> ui::Widget<S> for View2<R,S> {

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
        self.id
    }

    fn get_config(&self) -> ui::WidgetConfig
    {
        self.config.clone()
    }
}

impl<R, S:SceneT>  ui::gameview::GameViewTrait<S> for View2<R,S> {

    fn play(&mut self)
    {
        self.state = 1;
    }

    fn pause(&mut self)
    {
        self.state = 0;
    }

    fn stop(&mut self)
    {
        println!("TODO gameview stop");
    }

    fn get_scene_id(&self) -> S::Id
    {
        self.scene_id.clone()
    }

    fn update(&mut self) -> bool {
        self.input.clear();

        if self.state == 1 {
            self.request_update();
            true
        }
        else {
            false
        }
    }

    fn get_input(&self) -> &input::Input
    {
        &self.input
    }

    fn request_update(&self)
    {
        unsafe { ui::jk_glview_request_update(self.glview); }
    }
}

