use libc::{c_char, c_void, c_int, c_uint, size_t};
use std::mem;
use std::sync::{RwLock, Arc};
use std::ptr;
use std::rc::{Rc,Weak};
use std::cell::{RefCell};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::any::{Any};//, AnyRefExt};
use std::path::Path;
use std::fs;
use std::fs::File;
use serde_json;
use std::io::{Read,Write};
use std::ffi::{CString,CStr};
use uuid::Uuid;

use dormin::{vec, scene, object, component, render, resource};
use ui::{Tree,PropertyUser,View,EditView,Command,Action,
PropertyWidget,PropertyBox, PropertyId};
use ui;
use operation;

use uuid;
use dragger;
use util;
use util::Arw;
use data::Data;
use data::ToId;

use state::State;
use data::{DataT, SceneT};
use ui::gameview::GameViewTrait;

#[repr(C)]
pub struct Window;
#[repr(C)]
pub struct Evas_Object;
#[repr(C)]
pub struct Ecore_Animator;

pub type RustCb = extern fn(data : *mut c_void);
pub type RenderFunc = extern fn(data : *const c_void);
pub type ResizeFunc = extern fn(data : *const c_void, w : c_int, h : c_int);

pub type RenderFuncTmp = extern fn(data : *const c_void);
pub type ResizeFuncTmp = extern fn(data : *const c_void, w : c_int, h : c_int);

pub type PanelGeomFunc = extern fn(
    object : *const c_void,
    x : c_int,
    y : c_int,
    w : c_int,
    h : c_int);

pub type AnimatorCallback = extern fn(
    data : *const c_void) -> bool;

pub type ButtonCallback = extern fn(
    data : *const c_void);

pub type EntryCallback = extern fn(
    data : *const c_void,
    text : *const c_char
    );

pub type SelectCallback = extern fn(
    data : *const c_void,
    name : *const c_char);

type MonitorCallback = extern fn(
    data : *const c_void,
    path : *const c_char,
    event : i32);

pub type KeyDownFunc = extern fn(
    data : *const c_void,
    modifier : c_int,
    keyname : *const c_char,
    key : *const c_char,
    keycode : c_uint,
    timestamp : c_int);

/*
        init_cb: extern fn(*mut View),// -> (),
        draw_cb: extern fn(*mut View), // -> (),
        resize_cb: extern fn(*mut View, w : c_int, h : c_int) -> (),
        render: *const View
        */

#[link(name = "joker")]
extern {
    pub fn elm_simple_window_main();
    pub fn window_new(w : c_int, h : c_int) -> *const Window;
    pub fn jk_window_new(cb : RustCb, cb_data : *const c_void) -> *const Evas_Object;
    pub fn jk_glview_new(
        win : *const Evas_Object,
        data : *const c_void,
        init : RenderFunc,
        draw : RenderFunc,
        resize : ResizeFunc,
        key : KeyDownFunc
        ) -> *const ui::JkGlview;
    pub fn jk_window_request_update(win : *const Window);

    pub fn tmp_func(
        window: *const Window,
        data : *const c_void,
        init : RenderFuncTmp,
        draw : RenderFuncTmp,
        resize : ResizeFuncTmp
        );
    //fn window_button_new(window : *const Window);
    pub fn window_callback_set(
        window : *const Window,
        data: *const c_void,
        mouse_down : extern fn(
            data : *const c_void,
            modifier : c_int,
            button : c_int,
            x : c_int,
            y : c_int,
            timestamp : c_int
            ),
        mouse_up : extern fn(
            data : *const c_void,
            modifier : c_int,
            button : c_int,
            x : c_int,
            y : c_int,
            timestamp : c_int
            ),
        mouse_move : extern fn(
            data : *const c_void,
            modifier : c_int,
            button : c_int,
            curx : c_int,
            cury : c_int,
            prevx : c_int,
            prevy : c_int,
            timestamp : c_int
            ),
        mouse_wheel : extern fn(
            data : *const c_void,
            modifier : c_int,
            direction : c_int,
            z : c_int,
            x : c_int,
            y : c_int,
            timestamp : c_int
            ),
        key_down : KeyDownFunc
        );

    pub fn init_callback_set(
        cb: extern fn(*const c_void) -> (),
        data: *const c_void
        ) -> ();
    pub fn exit_callback_set(
        cb: extern fn(*const c_void) -> (),
        data: *const c_void
        ) -> ();

    fn jk_list_wdg_new(win : *const Window, name : *const c_char) -> *const Evas_Object;
    fn jk_list_wdg_new2(win : *const Window, name : *const c_char) -> *const Evas_Object;
    fn elm_hover_target_set(hover : *const Evas_Object, target : *const Evas_Object);

    fn jk_list_fn_set(
        o : *const ui::Evas_Object,
        sel_callback: SelectCallback,
        data : *const c_void);

    //fn window_object_get(
    //    obj : *const Window) -> *const Evas_Object;

    fn evas_object_geometry_get(
        obj : *const Evas_Object,
        x : *mut c_int,
        y : *mut c_int,
        w : *mut c_int,
        h : *mut c_int);

    fn elm_object_part_text_set(
        obj : *const Evas_Object,
        part : *const c_char,
        text : *const c_char);

    pub fn evas_object_show(o : *const Evas_Object);
    pub fn evas_object_hide(o : *const Evas_Object);
    fn evas_object_move(o : *const Evas_Object, x : c_int, y : c_int);
    fn evas_object_resize(o : *const Evas_Object, w : c_int, h : c_int);


    fn jklist_set_names(o : *const Evas_Object, names : *const c_void, len : size_t);

    pub fn ecore_animator_add(cb : AnimatorCallback, data : *const c_void) -> *const Ecore_Animator;
    fn jk_monitor_add(cb : MonitorCallback, data : *const c_void, path : *const c_char);

    fn jk_panel_new(
      w : *const Window,
      x : c_int,
      y : c_int,
      width : c_int,
      height : c_int,
      mov : PanelGeomFunc,
      resize : PanelGeomFunc,
      data : *const c_void
      ) -> *const Evas_Object;

}

fn object_geometry_get(obj : *const Evas_Object) -> (i32, i32, i32, i32)
{
    let (mut x, mut y, mut w, mut h) : (c_int, c_int, c_int, c_int) = (5,6,7,8);
    unsafe { evas_object_geometry_get(obj, &mut x, &mut y, &mut w, &mut h); }
    (x, y, w, h)
}

fn elm_object_text_set(
        obj : *const Evas_Object,
        text : *const c_char)
{
    unsafe { elm_object_part_text_set(obj, ptr::null(), text); }
}

pub extern fn init_cb(data: *const c_void) -> () {
    let app_data : &AppCbData = unsafe { &*(data as *const AppCbData) };
    let container_arw = app_data.container.clone();

    let wc = WindowConfig::load();

    let mut views = create_views(container_arw.clone(), &wc.views);
    init_views(container_arw.clone(), &wc, &mut views);

    {
        let container = &mut *container_arw.write().unwrap();
        container.views = views;
    }

    init_gameview(container_arw.clone(), &wc.gameview.clone().unwrap_or_default());

    let path = CString::new("shader".as_bytes()).unwrap();
    unsafe { jk_monitor_add(file_changed, Box::into_raw(box container_arw.clone()) as *const c_void, path.as_ptr()); }
}

fn create_views(container_arw : Arw<WidgetContainer>, views_config : &[ViewConfig]) -> Vec<Box<EditView<Scene2>>>
{
    let mut views = Vec::with_capacity(views_config.len());

    for v in views_config {
        let container = &mut *container_arw.write().unwrap();
        let render = box render::Render::new(&container.data.factory, container.resource.clone());

        let view = box View::new(
            container.resource.clone(),
            render,
            v.window.w,
            v.window.h,
            v.camera.clone());

        views.push(view as Box<EditView<Scene2>>);
        let scene = if let Some(ref scene) = v.scene {
            container.data.get_or_load_scene(scene.as_str())
        }
        else {
            container.data.get_or_load_any_scene()
        }.clone();

        container.set_scene(scene);
    }

    views
}

fn init_views<S:SceneT>(container_arw : Arw<WidgetContainer>, wc : &WindowConfig, views : &mut [Box<EditView<S>>])
{
    for (i,v) in views.iter_mut().enumerate() {
        let v : &mut Box<EditView<S>> = v;

        let pc = wc.property.clone();
        let tc = wc.tree.clone().unwrap_or_default();

        let win = unsafe { ui::window_new(v.get_config().w, v.get_config().h) };

        //TODO remove from here?
        init_property(&container_arw, win, &pc);
        init_tree(&container_arw, win, &tc);
        init_action(&container_arw, win, v.get_id());

        {
        //let container = &mut *app_data.container.write().unwrap();
        //container.list.create(win);

        //app_data.container.write().unwrap().list.create(win);
        container_arw.write().unwrap().list.create(win);
        }

        let wcb = ui::WidgetCbData::with_index(&container_arw, i);
        v.init(win, wcb);
    }

}

fn init_property(container : &Arw<WidgetContainer>, win : *const Window, pc : &WidgetPanelConfig)
{
    let container_arw = container.clone();
    let container = &mut *container.write().unwrap();

    container.property.config = pc.clone();
    container.property.create(win);

    let p = Rc::new(ui::PropertyBox::new(&*container.property));
    let pd = ui::WidgetCbData::new_with_widget(&container_arw, p.clone());

    unsafe {
        ui::property::jk_property_cb_register(
            ui::property_box::property_box_cb_get(p.jk_property),
            Box::into_raw(box pd) as *const c_void,
            ui::property_list::changed_set_float,
            ui::property_list::changed_set_string,
            ui::property_list::changed_set_enum,
            ui::property_list::register_change_string,
            ui::property_list::register_change_float,
            ui::property_list::register_change_enum,
            ui::property_list::register_change_option,
            ui::property_list::expand,
            ui::property_list::contract,
            ui::property::vec_add,
            ui::property::vec_del);
    }

    container.property.widget = Some(p);
}

fn init_tree(container : &Arw<WidgetContainer>, win : *const Window, tree_config : &WidgetConfig)
{
    let container_arw = container.clone();
    let container = &mut *container.write().unwrap();

    let mut t = box ui::Tree::new(win, tree_config);
    let tsd = ui::WidgetCbData::with_index(&container_arw, 0);

    unsafe {
        ui::tree::tree_register_cb(
            t.jk_tree,
            Box::into_raw(box tsd) as *const c_void,
            ui::tree::name_get,
            ui::tree::item_selected,
            ui::tree::can_expand,
            ui::tree::expand,
            ui::tree::selected,
            ui::tree::unselected,
            //TODO remove panel_move
            ui::tree::panel_move,
            );
    }

    match container.state.context.scene {
        Some(ref s) => {
            println!("TODO chris, scene tree, {}, {}", file!(), line!());
            //let sb = &*s.borrow();
            //t.set_scene(sb);
        },
        None => {
        }
    }

    container.tree = Some(t);
}

fn init_action(container : &Arw<WidgetContainer>, win : *const Window, view_id : Uuid)
{
    let container_arw = container.clone();
    let container = &mut *container.write().unwrap();

    let mut menu = box ui::Action::new(win, ui::action::Position::Top, view_id);

    let mut a = box ui::Action::new(win, ui::action::Position::Bottom, view_id);
    let command = box ui::Command::new(win);

    let ad = ui::WidgetCbData::with_ptr(&container_arw, unsafe { mem::transmute(&*a)});


    a.add_button("new scene", ui::action::scene_new, ad.clone());
    a.add_button("add empty", ui::action::add_empty, ad.clone());
    {
    a.add_button_closure("add empty closure", move || {
        println!("add button from closure!!!!!!!!!!!!, it's awesome !!!!!!!");
        let cont = &mut *container_arw.write().unwrap();
        ui::add_empty(cont, view_id);
    });
    }
    a.add_button(
        "open game view",
        ui::action::open_game_view,
        ad.clone());
    a.add_button(
        "pause",
        ui::action::pause_scene,
        ad.clone());
    a.add_button(
        "play",
        ui::action::play_scene,
        ad.clone());

    a.add_button(
        "compile_test",
        ui::action::compile_test,
        ad.clone());

    let name = match container.state.context.scene {
        Some(ref s) => {
            s.get_name()
        },
        None => {
            String::from("none")
        }
    };

    menu.add_button(">", ui::action::scene_list, ad.clone());
    menu.add_entry(String::from("scene"),&name, ui::action::scene_rename, ad.clone());
    menu.add_button("+", ui::action::scene_new, ad.clone());

    container.action = Some(a);
    container.command = Some(command);
    container.menu = Some(menu);

    //container.list.create(w);
}

fn init_gameview(container_arw : Arw<WidgetContainer>, gameview_config : &WidgetConfig)
{
    let op_scene = {
        let container = &mut *container_arw.write().unwrap();
        container.can_create_gameview()
     };

    if let Some(scene) = op_scene {

        let gv = create_gameview_window(
            container_arw.clone(),
            scene,
            &gameview_config);

        let container = &mut *container_arw.write().unwrap();
        container.set_gameview(gv);

        println!("ADDDDDDDD animator");
        unsafe {
            //ui::ecore_animator_add(ui::update_play_cb, mem::transmute(wcb.container));
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WidgetConfig
{
    pub visible : bool,
    pub x : i32,
    pub y : i32,
    pub w : i32,
    pub h : i32,
}

impl WidgetConfig
{
    pub fn new_from_obj(obj : *const Evas_Object) -> WidgetConfig
    {
        let (x, y, w, h) = object_geometry_get(obj);

        WidgetConfig {
            x : x,
            y : y,
            w : w,
            h : h,
            visible : true
        }
    }

    pub fn new() -> WidgetConfig
    {
        WidgetConfig::with_width_height(300,400)
    }

    pub fn with_width_height(w : i32, h : i32) -> WidgetConfig
    {
        WidgetConfig {
            x : 10,
            y : 10,
            w : w,
            h : h,
            visible : true
        }
    }
}

impl Default for WidgetConfig
{
    fn default() -> WidgetConfig 
    {
        WidgetConfig::new()
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ViewConfig
{
    window : WidgetConfig,
    scene : Option<String>,
    camera : ui::view::CameraView,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WindowConfig
{
    views: Vec<ViewConfig>,
    property : WidgetPanelConfig,
    tree : Option<WidgetConfig>,
    gameview : Option<WidgetConfig>
}

impl WindowConfig {

    fn new(c : &WidgetContainer) ->  WindowConfig
    {
        let mut wc = WindowConfig {
            views : Vec::new(),

            property : c.property.config.clone(),
            tree : match c.tree {
                None => None,
                Some(ref t) => Some(t.get_config())
            },
            gameview : match c.gameview {
                None => None,
                Some(ref t) => Some(t.get_config())
            }
        };

        for v in &c.views {
            let vc = ViewConfig {
                window : v.get_config().clone(),
                scene : match c.state.context.scene {
                    Some(ref s) => {
                        Some(s.get_name())
                    },
                    None => None
                },
                camera : v.get_camera().clone()
            };
            wc.views.push(vc);
        }

        wc
    }

    fn default() ->  WindowConfig
    {
        let mut wc = WindowConfig {
            views : Vec::new(),
            property : WidgetPanelConfig::default(),
            tree : None,
            gameview : None
        };

        /*

        let vc = ViewConfig {
            //window : WidgetConfig::new( unsafe { window_object_get(win) })
            window : WidgetConfig{
                x : 0,
                y : 0,
                w : 800,
                h : 500,
                visible : true
            },
            scene : None,
        };

        wc.views.push(vc);

        */
        wc
    }


    fn save(&self)
    {
        println!("save scene todo serialize");
        //let path : &Path = self.name.as_ref();
        let path : &Path = Path::new("windowconf");
        let mut file = File::create(path).ok().unwrap();
        let s = serde_json::to_string_pretty(self).unwrap();
        let result = file.write(s.as_bytes());
    }

    fn load() -> WindowConfig
    {
        let mut file = String::new();
        let mut wc : WindowConfig = match File::open(&Path::new("windowconf")){
            Ok(ref mut f) => {
                f.read_to_string(&mut file).unwrap();
                serde_json::from_str(&file).unwrap()
            },
            _ => {
                WindowConfig::default()
            }
        };

        if wc.views.is_empty() {
            wc.views.push(ViewConfig::default());
        }

        wc
    }

}

pub extern fn exit_cb(data: *const c_void) -> ()
{
    let app_data : &AppCbData = { let d = data as *const AppCbData; unsafe {&*d}};
    let container = &mut *app_data.container.write().unwrap();

    if let Some(ref s) = container.state.context.scene {
        println!("going to save: {}", s.get_name());
        s.save();
    }

    let wc = WindowConfig::new(&*container);
    wc.save();
}

pub trait Widget
{
    fn handle_change(&self, change : operation::Change<Id>)
    {
        println!("please implement me");
    }

    fn set_visible(&mut self, b : bool)
    {
        println!("please implement me");
    }

    fn handle_change_prop(&self, prop_user : &PropertyUser , name : &str)
    {
        println!("implement handle_change_prop 7777777777777777");
    }

    fn get_id(&self) -> Uuid;

    fn get_config(&self) -> ui::WidgetConfig
    {
        ui::WidgetConfig::default()
    }
}

pub struct WidgetPanel<T>
{
    pub config : WidgetPanelConfig,
    pub widget : Option<Rc<T>>,
    pub eo : *const Evas_Object
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WidgetPanelConfig
{
    visible : bool,
    x : i32,
    y : i32,
    w : i32,
    h : i32,
}

impl Default for WidgetPanelConfig
{
    fn default() -> WidgetPanelConfig
    {
        WidgetPanelConfig {
            visible : true,
            x : 0i32,
            y : 0i32,
            w : 100i32,
            h : 400i32,
        }
    }
}

impl<T> WidgetPanel<T>
{
    pub fn new(config : WidgetPanelConfig, widget : Option<Rc<T>>) -> WidgetPanel<T>
    {
        WidgetPanel {
            config : config,
            widget : widget,
            eo : ptr::null()
        }
    }

    pub fn create(&mut self, win : *const Window)
    {
        self.eo = unsafe { jk_panel_new(
                win,
                self.config.x,
                self.config.y,
                self.config.w,
                self.config.h,
                panel_move,
                panel_move,
                mem::transmute(&self.config)
                )
        };

        let is_visible = self.config.visible;
        self.set_visible(is_visible);
    }

    pub fn visible(&self) -> bool
    {
        self.config.visible
    }

    pub fn set_visible(&mut self, b : bool)
    {
        self.config.visible = b;
        if b {
            unsafe { evas_object_show(self.eo); }
        }
        else {
            unsafe { evas_object_hide(self.eo); }
        }
    }
}

extern fn panel_move(
    data : *const c_void,
    x : c_int, y : c_int, w : c_int, h : c_int)
{
    let mut config : &mut WidgetPanelConfig  = unsafe {mem::transmute(data)};

    config.x = x;
    config.y = y;
    config.w = w;
    config.h = h;
}

//#[derive(Clone)]
pub struct Core
{
    //pub state : Arw<State>,
    pub container : Box<ui::WidgetContainer>
}

impl Core {
    fn new() -> Core {

        Core {
            container : box ui::WidgetContainer::new() 
        }
    }
}

pub struct ListWidget
{
    object : Option<*const Evas_Object>,
    entries : Vec<*const c_char>
}

impl ListWidget
{
    pub fn create(&mut self, win : *const Window)
    {
        let name = CString::new("xaca".as_bytes()).unwrap().as_ptr();
        //self.object = Some(unsafe { jk_list_wdg_new(win, name) });
        self.object = Some(unsafe { jk_list_wdg_new2(win, name) });
    }

    pub fn set_fn(&self, cb : SelectCallback, data : ui::WidgetCbData)
    {

        unsafe {
            if let Some(o) = self.object {
                jk_list_fn_set(o,
                           cb,
                           mem::transmute(box data));
            }
        }
    }

    fn show_list(&mut self, entries : Vec<String>, x : i32, y : i32)
    {
        if let Some(o) = self.object {
            unsafe {
                evas_object_show(o);
                evas_object_move(o, x, y);
                evas_object_resize(o, 150, 300);
            }

            self.show_list_common(o, entries);
        }
    }

    fn show_list_target(&mut self, entries : Vec<String>, target : *const Evas_Object)
    {
        if let Some(o) = self.object {
            unsafe {
                elm_hover_target_set(o, target);
                evas_object_show(o);
                //evas_object_move(o, x, y);
                //evas_object_resize(o, 150, 300);
            }

            self.show_list_common(o,entries);
        }
    }

    fn show_list_common(&mut self, obj : *const Evas_Object, entries : Vec<String>) 
    {
        let cs = util::string_to_cstring(entries);
        self.entries = cs.iter().map( |x| x.as_ptr()).collect();

        unsafe {
            jklist_set_names(obj, self.entries.as_ptr() as *const c_void, self.entries.len() as size_t);
        }
    }
}

/*
pub struct ControlContainer
{
    pub control : Box<Control>,
    pub context : Box<Context>
}
*/

//*
pub type Scene = Rc<RefCell<scene::Scene>>;
pub type Object = Arc<RwLock<object::Object>>;
pub type Id = uuid::Uuid;
//*/
/*
use dormin;
pub type Scene = dormin::world::World;
pub type Object = usize;
pub type Id = usize;
*/

pub type Scene2 = Scene;
pub type Object2 = Object;
pub type Id2 = Id;


pub struct WidgetContainer
{
    pub tree : Option<Box<Tree>>,
    pub property : Box<WidgetPanel<PropertyBox>>,
    pub command : Option<Box<Command>>,
    pub action : Option<Box<Action>>,
    pub views : Vec<Box<EditView<Scene2>>>,
    //pub views : Vec<Box<View>>,
    pub gameview : Option<Box<GameViewTrait<Scene2>>>,
    pub menu : Option<Box<Action>>,

    pub list : Box<ListWidget>,
    pub name : String,
    pub visible_prop : HashMap<Id, Weak<Widget>>,
    pub anim : Option<*const Ecore_Animator>,

    pub data : Box<Data<Scene2>>,
    pub resource : Rc<resource::ResourceGroup>,
    pub state : State<Scene2>
}

impl WidgetContainer
{
    pub fn new() -> WidgetContainer
    {
        WidgetContainer {
            tree : None,
            property : box WidgetPanel::new(WidgetPanelConfig::default(), None),
            //property : None,
            command : None,
            action : None,
            menu : None,
            views : Vec::new(),
            gameview : None,
            list : box ListWidget { object : None, entries : Vec::new() },
            name : String::from("yoplaboum"),
            visible_prop : HashMap::new(),
            anim : None,

            data : box Data::new(),
            resource : Rc::new(resource::ResourceGroup::new()),
            state : State::new()

        }
    }

    pub fn handle_change(&mut self, change : &operation::Change<Id2>, widget_origin: uuid::Uuid)
    {
        //if *change == operation::Change::None {
        if let operation::Change::None = *change {
            return;
        }

        match *change {
            operation::Change::DirectChange(ref name) => {
                println!("hangle change DIRECT");
                let o = match self.get_selected_object() {
                    Some(ob) => ob,
                    None => {
                        println!("direct change, no objetcs selected");
                        return;
                    }
                };

                if name == "name" {
                    if let Some(ref t) = self.tree {
                        if widget_origin != t.id {
                            t.update_object(&o.to_id());
                        }
                    };
                }

                //TODO remove
                /*
                let ups = must_update(&o, name);
                for up in &ups {
                    if let ui::ShouldUpdate::Mesh = *up {
                        let mut ob = o.write().unwrap();
                        let omr = ob.get_comp_data_value::<component::mesh_render::MeshRender>();
                        if let Some(ref mr) = omr {
                            ob.mesh_render = Some(mr.clone());
                        }
                    }
                }
                */

                if let Some(ref p) = self.property.widget {
                    if widget_origin != p.id {
                        if let Some(pu) = self.data.get_property_user_copy(o.to_id()) {
                             p.update_object_property(&*pu.as_show(), name);
                         }
                    }
                }
            },
            operation::Change::Objects(ref name, ref id_list) => {
                println!("hangle change OBJECTS :: {}",name);
                let sel = self.get_selected_object();
                for id in id_list {

                    
                    if name == "name" {
                        if let Some(ref t) = self.tree {
                            if widget_origin != t.id {
                                //TODO tree
                                println!("TODO, {}, {}", file!(), line!());
                                //t.update_object(id);
                            }
                        };
                    }
                    //else if name.starts_with("comp_data/MeshRender") {
                    else {
                        check_mesh(name, self, *id);
                    }

                    if let Some(ref o) = sel {
                        if *id == o.to_id()  {
                            if let Some(ref mut p) = self.property.widget {
                                if widget_origin != p.id {
                                    println!("hangle change, calling update objects");
                                    if let Some(pu) = self.data.get_property_user_copy(*id) {
                                        p.update_object_property(&*pu.as_show(), name);
                                    }
                                }
                            }
                        }
                    }
                }
            },
            operation::Change::VecAdd(ref id_list, ref name, index) =>
            {
                println!("vec add add add add : {}", index);
                let sel = self.get_selected_object();
                for id in id_list {

                    check_mesh(name, self, *id);
                    if let Some(ref o) = sel {
                        if *id == o.to_id()  {
                            if let Some(ref mut p) = self.property.widget {
                                //if widget_origin != p.id 
				{
                                    println!("update object property, this needs more info than just update the value, must indicate it is a vec change.
                                             so we dont remove and add all children again, and so the scroller doesnt make big jump");
                                    //p.update_object(&*ob, "");
                                    if let Some(pu) = self.data.get_property_user_copy(*id) {
                                    p.vec_add(pu.as_show(), name, index);
                                    }
                                }
                            }
                        }
                    }
                }
            },
            operation::Change::VecDel(ref id_list, ref name, index) =>
            {
                println!("vec del del del del");
                let sel = self.get_selected_object();
                for id in id_list {

                    check_mesh(name, self, *id);
                    if let Some(ref o) = sel {
                        if *id == o.to_id()  {
                            if let Some(ref mut p) = self.property.widget {
                                if widget_origin != p.id {
                                    println!("update object property, this needs more info than just update the value, must indicate it is a vec change.
                                             so we dont remove and add all children again, and so the scroller doesnt make big jump");
                                    if let Some(pu) = self.data.get_property_user_copy(*id) {
                                    p.vec_del(pu.as_show(), name, index);
                                    }
                                }
                            }
                        }
                    }
                }
            },
            operation::Change::ComponentChanged(id, ref comp_name) => {
                println!("comp changed : {} ", comp_name);
                let sel = self.get_selected_object();
                if let Some(ref o) = sel {
                    if id == o.to_id()  {
                        if let Some(ref mut p) = self.property.widget {
                            if widget_origin != p.id {
                                if let Some(pu) = self.data.get_property_user_copy(id) {
                                    p.update_object(pu.as_show(), "");
                                }
                            }
                        }
                    }
                }

                println!("TODO MeshRender update code commented, remove? {}, {}", file!(), line!());
                /*
                if comp_name.starts_with("MeshRender") {
                    if let Some(ref scene) = self.get_scene() {
                        let oob = scene.find_object_by_id(id);

                        if let Some(o) = oob {
                            println!("please update mesh");
                            let omr = scene.get_comp_data_value::<component::mesh_render::MeshRender>(o.clone());
                            if let Some(ref mr) = omr {
                                let mut ob = o.write().unwrap();
                                ob.mesh_render = Some(mr.clone());
                            }
                        }
                    };
                }
                */
            },
            operation::Change::SceneRemove(ref id, ref parents, ref obs) => {
                {
                    println!("container, sceneremove!!!!!!!!");
                    self.state.context.remove_objects_by_id(obs);
                }
                if let Some(ref mut t) = self.tree {
                    if widget_origin != t.id {
                        t.remove_objects_by_id(obs.clone());
                    }
                }
                //TODO
                println!("do something for the other widget");
                self.handle_event(Event::SelectedChange, widget_origin);
            },
            operation::Change::SceneAdd(ref id, ref parents, ref obs) => {
                let scene = match self.get_scene() {
                    Some(s) => s,
                    None => return
                };

                let objects = scene.find_objects_by_id(&mut obs.clone());

                // todo
                match self.tree {
                    Some(ref mut t) => {
                        if widget_origin != t.id {
                            //TODO remove Some
                            //let p : Vec<Option<Id>> = parents.iter().map(|x| Some(*x)).collect();
                            let n : Vec<String> = objects.iter().map(|o| scene.get_object_name(o.clone())).collect();
                            let has_children : Vec<bool> = objects.iter().map(|o| !scene.get_children(o.clone()).is_empty()).collect();
                            t.add_objects(parents, &objects, n, &has_children);
                        }
                    },
                    None => {
                        println!("control no tree");
                    }
                }
            },
            operation::Change::DraggerOperation(ref op) => {
                self.handle_dragger_operation(op);
            },
            operation::Change::PropertyId(id, ref name) => {
                self.handle_change_new_id(widget_origin, id, name);
            },
            _ => {}
        }

        self.update_all_views();
    }

    fn handle_dragger_operation(&mut self, op : &dragger::Operation) {
        let (prop, operation) = {
            let context = &self.state.context;;
            match *op {
                dragger::Operation::Translation(v) => {
                    let prop = vec!["position".to_owned()];
                    let cxpos = &self.state.saved_positions;;
                    let mut saved_positions = Vec::with_capacity(cxpos.len());
                    for p in cxpos {
                        saved_positions.push((box *p ) as Box<Any>);
                    }
                    let mut new_pos = Vec::with_capacity(cxpos.len());
                    for p in cxpos {
                        let np = *p + v;
                        new_pos.push((box np) as Box<Any>);
                    }
                    let change = operation::OperationData::Vector(
                        saved_positions,
                        new_pos);

                    (prop, change)
                },
                dragger::Operation::Scale(v) => {
                    let prop = vec!["scale".to_owned()];
                    let cxsc = self.state.saved_scales.clone();
                    let mut saved_scales = Vec::with_capacity(cxsc.len());
                    for p in &cxsc {
                        saved_scales.push((box *p ) as Box<Any>);
                    }
                    let mut new_sc = Vec::with_capacity(cxsc.len());
                    for s in &cxsc {
                        let ns = *s * v;
                        new_sc.push((box ns) as Box<Any>);
                    }
                    let change = operation::OperationData::Vector(
                        saved_scales,
                        new_sc);

                    (prop, change)
                },
                dragger::Operation::Rotation(q) => {
                    let prop = vec!["orientation".to_owned(), "*".to_owned()];
                    let cxoris = self.state.saved_oris.clone();
                    let mut saved_oris = Vec::with_capacity(cxoris.len());
                    for p in &cxoris {
                        saved_oris.push((box *p ) as Box<Any>);
                    }
                    let mut new_ori = Vec::with_capacity(cxoris.len());
                    for p in &cxoris {
                        let no = *p * q;
                        new_ori.push((box no) as Box<Any>);
                    }
                    let change = operation::OperationData::Vector(
                        saved_oris,
                        new_ori);

                    (prop, change)
                }
            }
        };
        self.state.request_operation(prop, operation, &mut *self.data);
        //let op = self.state.make_operation(prop, operation);
        //self.state.op_mgr.add_with_trait2(box op);
        //self.state.op_mgr.redo(self.data)
    }

    pub fn handle_event(&mut self, event : Event<Object>, widget_origin: uuid::Uuid)
    {
        match event {
            Event::SelectObject(ob) => {
                let mut l = vec![ob.to_id()];
                self.state.context.select_by_id(&mut l);
                self.handle_event(Event::SelectedChange, widget_origin);
            },
            Event::UnselectObject(ob) => {
                let v = vec![ob.to_id()];
                self.state.context.remove_objects_by_id(&v);
                self.handle_event(Event::SelectedChange, widget_origin);
            },
            Event::DraggerTranslation(t) => {
                //TODO instead of this : 
                let change = self.state.request_translation(t);
                self.handle_change(&change, widget_origin);
                //TODO we should do this:
                //let wanted_change = self.state.request_change_from_event(event)
                //if self.data.apply_change(wanted_change) {
                //  self.ui.reflect_change(wanted_change); //or something else than wanted_change
                //}
            },
            Event::DraggerOperation(ref o) => {
                self.handle_dragger_operation(o);
            },
            Event::DraggerScale(s) => {
                let change = self.state.request_scale(s);
                self.handle_change(&change, widget_origin);
            },
            Event::DraggerRotation(r) => {
                let change = self.state.request_rotation(r);
                self.handle_change(&change, widget_origin);
            },
            Event::ChangeSelected(ref list) => {
                self.state.context.selected = list.clone();
                self.handle_event(Event::SelectedChange, widget_origin);
            },
            Event::SelectedChange => {
                let sel = &self.state.context.selected;

                if let Some(ref mut t) = self.tree {
                    if widget_origin != t.id {
                        let ids = self.state.context.get_vec_selected_ids();
                        t.select_objects(ids);
                    }
                }
                println!("selected changed");

                if sel.is_empty() {
                    if let Some(ref mut p) = self.property.widget {
                        if let Some(ref s) = self.state.context.scene {
                            //p.set_scene(&*s.borrow());
                            //p.set_prop_cell(s.clone(), "scene");
                            p.set_current_id(s, s.to_id(), "scene");
                        }
                    }
                }
                else if sel.len() != 1 {
                    if let Some(ref mut p) = self.property.widget {
                        if widget_origin != p.id {
                            p.set_nothing();
                        }
                    }
                    else {
                        println!("container no property");
                    }
                }
                else {
                    let opid = sel.get(0).map(|x| x.to_id()); 

                    //if let Some(o) = sel.get(0) {
                    if let Some(oid) = opid  {
                        if let Some(ref mut p) = self.property.widget {
                            if widget_origin != p.id {
                                println!("STUFF");
                                if let Some(ppp) = self.data.get_property_user_copy(oid) {
                                p.set_prop(&*ppp, oid, "object");
                            }
                                self.visible_prop.insert(
                                        oid, Rc::downgrade(p) as Weak<Widget>);
                            }
                        }
                        else {
                            println!("container has no property");
                        }
                    }
                }
            },
            Event::Undo => {
                let change = self.state.undo(&mut *self.data);
                self.handle_change(&change, widget_origin);
            },
            Event::Redo => {
                let change = self.state.redo(&mut *self.data);
                self.handle_change(&change, widget_origin);
            },
            Event::CameraChange => {
                self.update_view(widget_origin);
            },
            _ => {}
        }
    }

    fn get_scene(&self) -> Option<Scene>
    {
        self.state.context.scene.clone()
    }

    pub fn find_view(&self, id : Uuid) -> Option<&EditView<Scene>>
    {
        for v in &self.views
        {
            if v.get_id() == id {
                return Some(&**v)
            }
        }
        None
    }

    pub fn play_gameview(&mut self) -> bool
    {
        if let Some(ref mut gv) = self.gameview {
            gv.play();
            true
        }
        else {
            false
        }
    }

    pub fn open_gameview(&mut self) -> bool
    {
        if let Some(ref mut gv) = self.gameview {
            gv.set_visible(true);
            true
        }
        else {
            false
        }
    }

    fn can_create_gameview(&mut self) -> Option<Scene>
    {
        if self.gameview.is_some() {
            return None;
        }

        let scene = if let Some(ref mut s) = self.state.context.scene {
            let mut scene = s.clone();
            scene.init_for_play(&self.resource);
            scene
        }
        else {
            return None;
        };

        Some(scene)
    }

    pub fn set_gameview(&mut self, gv : Box<GameViewTrait<Scene>>)
    {
        let gvo = &mut self.gameview;
        if gvo.is_some() {
            //panic!("cannot start animator");
            return;
        }

        *gvo = Some(gv);
    }

    pub fn update_play(&mut self) -> bool
    {
        if let Some(ref mut gv) = self.gameview {
            let id = gv.get_scene_id();
            if let Some(scene) = self.data.get_scene_mut(id) {
                scene.update(0.01f64, gv.get_input(), &*self.resource);
            }
            let was_updated = gv.update();

            if was_updated {
                for view in &self.views {
                    view.request_update();
                }
            }
            true
        }
        else {
            false
        }
    }

    pub fn handle_change_new(&self, widget_id : Uuid, p : &PropertyUser, name : &str)
    {
        let pid = p.get_id();

        println!("handle change new 00 ");

        if let Some(w) = self.visible_prop.get(&pid) {
        println!("handle change new 11 ");

            if let Some(w) = w.upgrade() {
                if w.get_id() == widget_id {
                    println!("same id as the widget so get out (but right now the continue is commented)");
                    //continue;
                }

        println!("handle change new 22 ");

                w.handle_change_prop(p, name);
            }
        }

        if name == "name" {
            if let Some(ref tree) = self.tree {
                tree.handle_change_prop(p, name);
            }
        }
    }

    pub fn handle_change_new_id(&self, widget_id : Uuid, pid : Id, name : &str)
    {
        if let Some(ppp) = self.data.get_property_user_copy(pid) {
            if let Some(w) = self.visible_prop.get(&pid) {

                if let Some(w) = w.upgrade() {
                    if w.get_id() == widget_id {
                        println!("same id as the widget so get out (but right now the continue is commented)");
                        //continue;
                    }

                    w.handle_change_prop(&*ppp, name);
                }
            }
            if name == "name" {
                if let Some(ref tree) = self.tree {
                    tree.handle_change_prop(&*ppp, name);
                }
            }
        }
    }

    fn update_all_views(&self)
    {
        for view in &self.views {
            view.request_update();
        }

        if let Some(ref gv) = self.gameview {
            gv.request_update();
        }
    }

    fn update_view(&self, id : uuid::Uuid)
    {
        for view in &self.views {
            if view.get_id() == id {
                view.request_update();
            }
        }
    }

    pub fn set_scene(&mut self, scene : Scene2)
    {
        if let Some(ref mut t) = self.tree {
            //TODO chris
            println!("TODO set scene tree {}, {}", file!(), line!());
            //t.set_scene(&scene.borrow());
        }

        if let Some(ref mut p) = self.property.widget {
            p.set_nothing();
        }

        if let Some(ref mut m) = self.menu {
            if let Entry::Occupied(en) = m.entries.entry(String::from("scene")) {
                elm_object_text_set(
                    unsafe {mem::transmute(*en.get())},
                    CString::new(scene.get_name().as_str()).unwrap().as_ptr());
            }
        }

        self.state.context.set_scene(scene);

        for view in &self.views {
            view.request_update();
        }
    }

    //fn get_selected_object(&self) -> Option<Arc<RwLock<object::Object>>>
    fn get_selected_object(&self) -> Option<Object>
    {
        self.state.get_selected_object()
    }
}

// TODO remove/rework
//OLD : Send to c with mem::transmute(box data)  and free in c
pub struct WidgetCbData
{
    pub container : Arw<WidgetContainer>,
    pub widget : *const c_void,
    pub object : Option<*const Evas_Object>,
    pub widget2 : Option<Rc<PropertyWidget>>,
    pub index : usize
}

impl Clone for WidgetCbData {
    fn clone(&self) -> WidgetCbData
    {
        WidgetCbData {
            container : self.container.clone(),
            widget : self.widget,
            object : self.object,
            widget2 : self.widget2.clone(),
            index : 0usize
        }
    }
}

impl WidgetCbData {
    pub fn with_ptr(c : &Arw<WidgetContainer>, widget : *const c_void) -> WidgetCbData
    {
        println!("TODO free me");
        WidgetCbData {
            container : c.clone(),
            widget : widget,
            object : None,
            widget2 : None,
            index : 0usize,
        }
    }

    pub fn with_index(c : &Arw<WidgetContainer>, index : usize) -> WidgetCbData
    {
        println!("TODO free me");
        WidgetCbData {
            container : c.clone(),
            widget : ptr::null(),
            object : None,
            widget2 : None,
            index : index,
        }
    }

    pub fn new_with_widget(c : &Arw<WidgetContainer>, widget : Rc<PropertyWidget> ) -> WidgetCbData
    {
        println!("TODO free me");
        WidgetCbData {
            container : c.clone(),
            widget : ptr::null(),
            object : None,
            widget2 : Some(widget),
            index : 0usize,
        }
    }


    pub fn new(c : &Arw<WidgetContainer>, widget : *const c_void) -> WidgetCbData
    {
        println!("TODO free me");
        WidgetCbData {
            container : c.clone(),
            widget : widget,
            object : None,
            widget2 : None,
            index : 0usize,
        }
    }

    pub fn with_ptr_obj(c : &Arw<WidgetContainer>, widget : *const c_void, object : *const Evas_Object) -> WidgetCbData
    {
        println!("TODO free me");
        WidgetCbData {
            container : c.clone(),
            widget : widget,
            object : Some(object),
            widget2 : None,
            index : 0usize,
        }
    }
}

pub struct AppCbData
{
    pub container : Arw<WidgetContainer>
}

impl Clone for AppCbData {
    fn clone(&self) -> AppCbData
    {
        AppCbData {
            container : self.container.clone()
        }
    }
}

impl AppCbData {
    pub fn new() -> AppCbData {
        AppCbData {
            container : Arc::new(RwLock::new(WidgetContainer::new()))
        }
    }
}


//TODO choose how deep is the event, like between those 3 things
pub enum Event<Object>
{
    KeyPressed(String),
    ViewKeyPressed(String),
    ShowTree(String),
    SelectedChange,
    SelectObject(Object),
    UnselectObject(Object),
    ChangeSelected(Vec<Object>),

    RectVisibleSet(bool),
    RectSet(f32, f32, f32, f32),
    CameraChange,

    DraggerClicked,
    DraggerOperation(dragger::Operation), //TODO check if remove
    DraggerTranslation(vec::Vec3),
    DraggerScale(vec::Vec3),
    DraggerRotation(vec::Quat),
    DraggerChange,

    Undo,
    Redo,

    Empty
}

pub fn add_empty(container : &mut WidgetContainer, view_id : Uuid)
{
    println!("add empty");

    let mut o = container.data.factory.create_object("new object");

    let position = if let Some(v) = container.find_view(view_id) {
        let (p,q) = v.get_camera().transform.get_pos_quat();
        p + q.rotate_vec3(&vec::Vec3::new(0f64,0f64,-100f64))
    }
    else {
        vec::Vec3::zero()
    };

    o.position = position;

    let s = if let Some(ref s) = container.state.context.scene {
        s.clone()
    }
    else {
        return;
    };

    let ao =  Arc::new(RwLock::new(o));
    let mut vec = Vec::new();
    vec.push(ao.clone());

    let parent = vec![None];

    let mut ops = Vec::new();
    let vs = Vec::new();
    let addob = container.state.request_operation(
            vs,
            operation::OperationData::SceneAddObjects(s.to_id(),parent,vec.clone()),
            &mut *container.data
            );

    ops.push(addob);

    for op in &ops {
        container.handle_change(op, view_id);
    }

    container.handle_event(ui::Event::ChangeSelected(vec), view_id);
}

pub fn scene_list(container : &Arw<WidgetContainer>, view_id : Uuid, obj : Option<*const Evas_Object>)
{
    let container_arw = container.clone();
    let container = &mut *container.write().unwrap();

    let files = util::get_files_in_dir("scene");
    let filesstring : Vec<String> = files.iter().map(|x| String::from(x.to_str().unwrap())).collect();

    let (x, y) = if let Some(o) = obj {
        println!("TODO show the list of scene, there is an obj");
        let (mut x, mut y, mut w, mut h) : (c_int, c_int, c_int, c_int) = (5,6,7,8);
        unsafe { evas_object_geometry_get(o, &mut x, &mut y, &mut w, &mut h); }
        container.list.show_list_target(filesstring, o);

        (x, y + h + 5)
    }
    else {
        println!("TODO show the list of scene, no obj");
        (250, 50)
    };

    //container.list.show_list(filesstring, x, y);

    let listwd = ui::WidgetCbData::new(&container_arw, unsafe { mem::transmute(&*container.list)});
    container.list.set_fn(select_list, listwd);
}

pub extern fn select_list(data : *const c_void, name : *const c_char)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();

    let s = unsafe {CStr::from_ptr(name)}.to_str().unwrap();
    println!("selection ..........{},  {}", container.name, s);
    let scene = container.data.get_or_load_scene(s).clone();
    container.set_scene(scene);
}

//TODO remove?
fn must_update(p : &ui::PropertyShow, path : &str) -> Vec<ui::ShouldUpdate>
{
    let vs: Vec<&str> = path.split('/').collect();

    let mut v = Vec::new();
    for i in &vs
    {
        v.push(i.to_string());
    }

    let mut r = Vec::new();

    while !v.is_empty() {
        let prop = ui::property::find_property_show(p, v.clone());
        if let Some(pp) = prop {
            r.push(pp.to_update())
        }
        else {
            println!("no property for : {:?}", v);
        }

        v.pop();
    }

    r
}

pub fn scene_rename(container : &mut WidgetContainer, widget_id : Uuid, name : &str)
{

    let s = if let Some(ref s) = container.state.context.scene {
        s.clone()
    }
    else {
        return;
    };

    let _ = fs::remove_file(s.get_name().as_str());

    s.set_name(String::from(name));
    s.save();

    /*
    let addob = container.request_operation(
            vs,
            operation::OperationData::SceneAddObjects(s.clone(),vec)
            );

    ops.push(addob);
    ops.push(operation::Change::ChangeSelected(list));

    for op in &ops {
        container.handle_change(op, view_id);
    }
    */
}

pub extern fn update_play_cb(container_data : *const c_void) -> bool
{
    let container_arw = container_data as *const Arw<ui::WidgetContainer>;
    let container_ref = unsafe { &*container_arw };
    let container = &mut container_ref.write().unwrap();
    container.update_play()
}

pub extern fn file_changed(
    data : *const c_void,
    path : *const c_char,
    event : i32)
{
    let s = unsafe {CStr::from_ptr(path)}.to_str().unwrap();
    let container_arw : &Arw<ui::WidgetContainer> = {
        let c = data as *const Arw<ui::WidgetContainer>;
        unsafe { &*c }
    };

    let container : &mut ui::WidgetContainer = &mut *container_arw.write().unwrap();

    let mut should_update_views = false;
    if s.ends_with(".frag") || s.ends_with(".vert") {
        println!("file changed : {}", s);
        let mut shader_manager = container.resource.shader_manager.borrow_mut();

        for shader in shader_manager.loaded_iter_mut() {
            let mut reload = false;
            if let Some(ref vert) = shader.vert_path {
                reload = vert == s;
            };

            if let Some(ref frag) = shader.frag_path {
                println!("FRAG : {}, {}", frag, s);
                reload = reload || frag == s;
            };

            if reload {
                if shader.reload() {
                    should_update_views = true;
                }
            }
        }
    }

    if should_update_views {
        container.update_all_views();
    }
}
extern fn gv_close_cb(data : *mut c_void) {
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(data)};
    let container : Box<Arw<ui::WidgetContainer>> = unsafe {mem::transmute(data)};
    let container = &mut *container.write().unwrap();
    if let Some(ref mut gv) = container.gameview {
        gv.set_visible(false);
    }
}


fn create_gameview_window(
    container : Arw<ui::WidgetContainer>,
    scene : Scene,
    config : &WidgetConfig
    ) -> Box<ui::gameview::GameViewTrait<Scene>>
{
    let win = unsafe {
        ui::jk_window_new(
            gv_close_cb,
            mem::transmute( box container.clone()))
    };

    unsafe { evas_object_resize(win, config.w, config.h); }

    let container : &mut ui::WidgetContainer = &mut *container.write().unwrap();

    let render = render::GameRender::new(container.resource.clone());

    /*
    ui::gameview::GameView::new(
        win,
        scene.borrow().id,
        &container.data as *const Box<Data<Scene>>,
        box render,
        config.clone())
        */

        //*
    ui::view2::View2::new(
        win,
        &*container.data as *const DataT<Scene>,
        config.clone(),
        render,
        scene.to_id(),
        )
        //*/

}

fn check_mesh(name : &str, wc : &WidgetContainer, id : Id2)
{
    println!("TODO remove this check_mesh {}, {}", file!(), line!());
    /*
    if name.starts_with("comp_data") {
        println!("TODO, only update when it was a mesh.
                                 right now 'MeshRender' is not in the property path...,
                                 maybe do a property path like comp_data/2/[MeshRender]mesh...
                                 or check serde first");
        let scene = wc.get_scene();
        let oob = if let Some(ref sc) = scene {
            sc.find_object_by_id(id)
        } else {
            None
        };

        if let Some(o) = oob {
            let mut ob = o.write().unwrap();
            println!("please update mesh");
            let omr = ob.get_comp_data_value::<component::mesh_render::MeshRender>();
            if let Some(ref mr) = omr {
                ob.mesh_render = Some(mr.clone());
            }
            else {
                ob.mesh_render = None;
            }
        }
    }
    */
}

