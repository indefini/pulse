use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use libc::{c_char, c_void, c_int};
use std::mem;
use std::collections::{LinkedList};//,Deque};
use std::ptr;
use std::cell::{RefCell};
use std::rc::Weak;
use std::rc::Rc;
use uuid::Uuid;
use std::ffi::CString;
use std::ffi::CStr;
use std::str;

use ui::Window;
use ui;
use ui::Widget;
//use control::Control;
use operation;
use data::SceneT;

#[repr(C)]
pub struct JkCommand;

pub type CommandCallback = extern fn(
    //fn_data : *const c_void,
    data : *const c_void,
    name : *const c_char,
    );

#[link(name = "joker")]
extern {
    fn window_command_new(window : *const Window) -> *const JkCommand;
    fn command_new(
        command : *const JkCommand,
        name : *const c_char,
        data : *const c_void,
        button_callback : CommandCallback);
    fn command_clean(
        command : *const JkCommand);
    fn command_show(
        command : *const JkCommand);
}

pub struct Command
{
    name : String,
    jk_command : *const JkCommand,
}

impl Command
{
    pub fn new(
        window : *const Window)
        //-> Box<Command>
        -> Command
    {
        //let c = box Command {
        let c = Command {
            name : String::from("command_name"),
            jk_command : unsafe {window_command_new(window)},
        };

        c
    }

    pub fn show(&self)
    {
        unsafe { command_show(self.jk_command); }
    }

    pub fn add_ptr(
        &self,
        name : &str,
        cb : CommandCallback, data : *const c_void)
    {
        unsafe {
            command_new(
                self.jk_command,
                CString::new(name.as_bytes()).unwrap().as_ptr(),
                data,
                cb);
        }
    }

    pub fn clean(&self)
    {
        unsafe {
            command_clean(self.jk_command);
        }
    }
}

pub extern fn add_empty<S:SceneT>(data : *const c_void, name : *const c_char)
{
    println!("command ::: add empty");

    //let cd : &CommandData = unsafe {mem::transmute(data)};
    let wcb : & ui::WidgetCbData<S> = unsafe {mem::transmute(data)};
    let v : &ui::View = unsafe {mem::transmute(wcb.widget)};
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    let container : &mut ui::WidgetContainer<S> = &mut *wcb.container.write().unwrap();

    ui::add_empty(container, (v as &Widget<S>).get_id());
}

/*
pub extern fn remove_selected(data : *const c_void, name : *const c_char)
{
    let cd : &CommandData = unsafe {mem::transmute(data)};

    if cd.control.borrow_state() != BorrowState::Unused {
        println!("control already borrowed ");
        return;
    }

    let mut control = cd.control.borrow_mut();
    let o = control.remove_selected_objects();

    match cd.property.borrow_state() {
        BorrowState::Unused => {
            println!("todo remove selected, property");
            //TODO cd.property.borrow_mut().set_();
        },
        _ => {println!("cannot borrow property");}
    };

    match cd.tree.borrow_state() {
        BorrowState::Unused => {
            println!("todo remove selected, tree");
            //let mut t = cd.tree.borrow_mut();
            //t.add_object(o.clone());
            //t.select(&o.read().unwrap().id);
        }
        _ => {}
    }
}
*/

pub extern fn set_scene_camera(data : *const c_void, name : *const c_char)
{
    println!("command ::: set scene camera");
}

pub extern fn remove_selected2<S:SceneT+'static>(data : *const c_void, name : *const c_char) 
{
    let wcb : & ui::WidgetCbData<S> = unsafe {mem::transmute(data)};
    let v : &ui::View = unsafe {mem::transmute(wcb.widget)};
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    let container : &mut ui::WidgetContainer<S> = &mut *wcb.container.write().unwrap();

    println!("if borrow panic check this code, {}, {} ", file!(), line!());
    /*
    if v.control.borrow_state() != BorrowState::Unused {
        println!("control already borrowed, remove selected2 ");
        return;
    }
    */

    //let mut control = v.control.borrow_mut();
    let change = container.state.remove_selected_objects(&mut *container.data);

    container.handle_change(&change, (v as &Widget<S>).get_id());
}

pub extern fn copy_selected<S:SceneT+'static>(data : *const c_void, name : *const c_char)
{
    let wcb : & ui::WidgetCbData<S> = unsafe {mem::transmute(data)};
    let v : &ui::View = unsafe {mem::transmute(wcb.widget)};
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    let container : &mut ui::WidgetContainer<S> = &mut *wcb.container.write().unwrap();

    println!("if borrow panic check this code, {}, {} ", file!(), line!());
    /*
    if v.control.borrow_state() != BorrowState::Unused {
        println!("control already borrowed, copy selected ");
        return;
    }
    */

    //let mut control = v.control.borrow_mut();
    let change = container.state.copy_selected_objects(&mut *container.data);

    container.handle_change(&change, (v as &Widget<S>).get_id());
}


pub extern fn set_camera2<S:SceneT+'static>(data : *const c_void, name : *const c_char)
{
    let wcb : & ui::WidgetCbData<S> = unsafe {mem::transmute(data)};
    let v : &ui::View = unsafe {mem::transmute(wcb.widget)};
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    let container : &mut ui::WidgetContainer<S> = &mut *wcb.container.write().unwrap();

    println!("commnd set camera");
    let change = container.state.set_scene_camera(&mut *container.data);

    container.handle_change(&change, (v as &Widget<S>).get_id());
}

extern fn add_comp<S:SceneT+'static>(data : *const c_void, name : *const c_char)
{
    let s = unsafe {CStr::from_ptr(name).to_bytes()};
    let s = str::from_utf8(s).unwrap();
    println!("TODO add component : {}", s);

    let wcb : & ui::WidgetCbData<S> = unsafe {&* (data as *const ui::WidgetCbData<S>)};
    let container : &mut ui::WidgetContainer<S> = &mut *wcb.container.write().unwrap();

    let id = container.views[wcb.index].get_id();

    let change = container.state.add_component(s, &mut *container.data);
    container.handle_change(&change, id);
}

pub extern fn add_component<S:SceneT+'static>(data : *const c_void, name : *const c_char)
{
    let wcb : & ui::WidgetCbData<S> = unsafe {mem::transmute(data)};
    let container : &mut ui::WidgetContainer<S> = &mut *wcb.container.write().unwrap();

    let s = unsafe {CStr::from_ptr(name).to_bytes()};
    let s = str::from_utf8(s).unwrap();

    if let Some(ref cmd) = container.command {

        cmd.clean();

        for i in &S::get_existing_components() {
            cmd.add_ptr(i, ui::command::add_comp::<S>, data);
        }

        cmd.show();
    }
}


