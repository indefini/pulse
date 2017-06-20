use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int};
use std::mem;
use std::collections::{LinkedList};//,Deque};
use std::ptr;
use std::cell::{RefCell, BorrowState};
use std::rc::Weak;
use std::rc::Rc;
use uuid::Uuid;
use std::ffi::{CString, CStr};

use dormin::{camera, object, scene};
use ui::Window;
use ui::{ButtonCallback,EntryCallback};
use ui;
use dormin::resource;
use uuid;
use data;

#[repr(C)]
pub struct JkAction;
pub struct JkLabel;
pub struct JkEntry;


#[link(name = "joker")]
extern {
    fn window_action_new(window : *const Window) -> *const JkAction;
    fn window_action_new_up(window : *const Window) -> *const JkAction;
    fn action_button_new(
        action : *const JkAction,
        name : *const c_char,
        data : *const c_void,
        button_callback : ButtonCallback) -> *const ui::Evas_Object;
    fn action_button_new1(
        action : *const JkAction,
        name : *const c_char)-> *const ui::Evas_Object;

    fn btn_cb_set(
        o : *const ui::Evas_Object,
        button_callback: ButtonCallback,
        data : *const c_void);

    fn action_label_new(
        action : *const JkAction,
        name : *const c_char) -> *const ui::Evas_Object;

    fn jk_label_set(
        label : *const JkLabel,
        name : *const c_char);

    fn action_entry_new(
        action : *const JkAction,
        name : *const c_char,
        data : *const c_void,
        entry_callback : EntryCallback ) -> *const JkEntry;

    fn action_show(
        action : *const JkAction,
        b : bool);
}

pub struct Action
{
    name : String,
    jk_action : *const JkAction,
    visible : bool,
    view_id : uuid::Uuid,
    pub entries : HashMap<String, *const JkEntry>,
}

pub enum Position
{
    Top,
    Bottom
}

impl Action
{
    pub fn new(
        window : *const Window,
        pos : Position,
        view_id : uuid::Uuid)
        -> Action
    {
        Action {
            name : String::from("action_name"),
            jk_action : match pos {
                Position::Bottom => unsafe {window_action_new(window)},
                Position::Top => unsafe {window_action_new_up(window)}
            },
            visible : true,
            view_id : view_id,
            entries : HashMap::new(),
        }
    }

    pub fn add_button(&self, name : &str, cb : ButtonCallback, data : ui::WidgetCbData) -> *const ui::Evas_Object
    {
        unsafe 
        {
            let b = action_button_new1(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr());

            let mut data = data;
            data.object = Some(b);

            btn_cb_set(b,
                       cb,
                       Box::into_raw(box data) as *const c_void);
            b
        }
    }

    pub fn add_button_closure<F:'static>(&mut self, name : &str, f : F) -> *const ui::Evas_Object
        where F : Fn()
    {
        unsafe {
            let b = action_button_new1(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr());

            let bf = Box::new(f);

            extern fn wrapper<F>(f : *const c_void)
                where F: Fn() {
                    let closure_ptr = f as *const F;
                    let closure = unsafe { &*closure_ptr };
                    return closure();
                }
            
            btn_cb_set(b, wrapper::<F>, Box::into_raw(bf) as *const c_void);
            b
        }
    }


    pub fn add_button_ptr(
        &self,
        name : &str,
        cb : ButtonCallback, data : *const c_void)
    {
        unsafe {
            action_button_new(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr(),
                data,
                cb);
        }
    }

    pub fn add_label(&self, name : &str) -> *const ui::Evas_Object
    {
        unsafe {
            action_label_new(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr())
        }
    }

    pub fn add_entry(&mut self, key : String, name : &str, cb : EntryCallback, data : ui::WidgetCbData)
        -> *const JkEntry
    {
        let en = unsafe {
            action_entry_new(
                self.jk_action,
                CString::new(name.as_bytes()).unwrap().as_ptr(),
                mem::transmute(box data),
                cb)
        };
        self.entries.insert(key, en);
        en
    }

    pub fn set_visible(&mut self, b : bool)
    {
        self.visible = b;
        unsafe {
            action_show(self.jk_action, b);
        }
    }

    pub fn visible(&self) -> bool
    {
        self.visible
    }

}

pub extern fn add_empty(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();

    ui::add_empty(container, action.view_id);
}

pub extern fn scene_new(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();

    let name = container.data.create_scene_name_with_context(&*container.state.context);
    let scene = container.data.get_or_load_scene(&name).clone();
    container.set_scene(scene);
}

pub extern fn scene_list(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    //let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();

    ui::scene_list(&wcb.container, action.view_id, wcb.object);
}


pub extern fn scene_rename(data : *const c_void, name : *const c_char)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();

    let s = unsafe {CStr::from_ptr(name)}.to_str().unwrap();

    println!("todo scene rename to : {}", s);
    ui::scene_rename(container, action.view_id, s);
}

pub extern fn open_game_view(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    //let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();

    if container.open_gameview() {
        return;
    }

    //TODO I think we don't need this now that we create the gameview in ui::init_cb
    // check for a while and then erase 2017-05-06
    /*
    let scene = if let Some(scene) = container.can_create_gameview() {
        scene
    }
    else {
        return;
    };

    let gv = ui::create_gameview_window(wcb.container.clone(), scene, &ui::WidgetConfig::new());

    container.set_gameview(gv);
    */
}

pub extern fn play_scene(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();

    if container.play_gameview() {
        if container.anim.is_none() {
            container.anim = Some( unsafe {
                ui::ecore_animator_add(ui::update_play_cb, Box::into_raw(box wcb.container.clone()) as *const c_void)
            });
        }
        return;
    }

    //TODO I think we don't need this now that we create the gameview in ui::init_cb
    // check for a while and then erase 2017-05-06
    /*
    let scene = if let Some(scene) = container.can_create_gameview() {
        scene
    }
    else {
        return;
    };

    let gv = ui::create_gameview_window(wcb.container.clone(), scene, &ui::WidgetConfig::new());
    container.set_gameview(gv);

    //println!("ADDDDDDDD animator");
    if container.anim.is_none() {
        container.anim = Some( unsafe {
            panic!("transmute is bad");
            ui::ecore_animator_add(ui::update_play_cb, mem::transmute(wcb.container.clone()))
        });
    }
    */
}

pub extern fn pause_scene(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();


    if let Some(ref mut gv) = container.gameview {
        gv.pause();
        //pause
    }
}

use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::fs;
extern crate libloading;
static mut libi : Option<i32> = None;

pub extern fn compile_test(data : *const c_void)
{
    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(data)};
    let action : &Action = unsafe {mem::transmute(wcb.widget)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();
    
    println!("compile test!!!!!!!!!!!!!!!!!!!!");
    
    //let suf = unsafe { libi += 1; libi };
    let old = unsafe { libi };
    let suf = unsafe {
        let val = libi.map_or(0, |v| v + 1); 
        libi = Some(val);
        val
    };

    thread::spawn(move || {
    
        let child = Command::new("cargo").arg("build")//.arg("--release")
        .current_dir("/home/chris/code/compload")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute");

        let output = child.wait_with_output().expect("failed with child");
        println!("this is the out output {}", String::from_utf8_lossy(&output.stdout));
        println!("this is the error output {}", String::from_utf8_lossy(&output.stderr));

        let path = "/home/chris/code/compload/target/debug/libcompload.so";
        let dest = "/home/chris/code/avion/libcompload.so".to_owned() + &suf.to_string();

        if let Some(o) = old { 
            let old_file = "/home/chris/code/avion/libcompload.so".to_owned() + &o.to_string();
            fs::remove_file(old_file);
        }

        fs::copy(path, &dest);

        

        //let lib = if let Ok(l) = libloading::Library::new("/home/chris/code/compload/target/release/libcompload.so") {
        let lib = if let Ok(l) = libloading::Library::new(dest) {
            l
        }
        else {
            println!("no lib");
            return;
        };

        unsafe {
            let fun  : libloading::Symbol<unsafe extern fn() ->i32> = if let Ok(f) = lib.get(b"get_my_i32") {
                f
            }
            else {
                return;
            };
            println!("{}",fun());

            let build_mesh : libloading::Symbol<unsafe extern fn(&mut ui::WidgetContainer, Uuid)> = if let Ok(f) = lib.get(b"build_mesh2") {
                f
            }
            else {
                return;
            };

            //build_mesh(container, action.view_id);
        }
    });
}


