use std::sync::{RwLock, Arc};
use std::collections::{HashMap,HashSet};
use libc::{c_char, c_void, c_int, c_float};
use std::str;
use std::mem;
use std::ptr;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::rc::Weak;
use std::any::{Any};//, AnyRefExt};
use std::ffi::CString;
use std::ffi::CStr;
use uuid;
use uuid::Uuid;

use ui::{Window, ButtonCallback};
use ui::{ChangedFunc, RegisterChangeFunc, PropertyTreeFunc, PropertyValue, PropertyConfig, PropertyUser,
PropertyShow, PropertyId, RefMut, Elm_Object_Item, ShouldUpdate, PropertyWidget, PropertyWidgetGen};
use ui;
use operation;
use data::SceneT;


use util::Arw;

#[repr(C)]
pub struct JkPropertyList;

extern {
    fn jk_property_list_new(
        window : *const Window,
        x : c_int,
        y : c_int,
        w : c_int,
        h : c_int
        ) -> *const JkPropertyList;

    fn property_list_clear(pl : *const JkPropertyList);

    fn property_list_group_add(
        pl : *const JkPropertyList,
        name : *const c_char
        );

    fn property_list_nodes_remove(
        pl : *const JkPropertyList,
        name : *const c_char
        );

    fn property_list_single_item_add(
        ps : *const JkPropertyList,
        container: *const PropertyValue,
        ) -> *const PropertyValue;

    fn property_list_single_vec_add(
        ps : *const JkPropertyList,
        container: *const PropertyValue,
        is_node : bool
        ) -> *const PropertyValue;

    fn property_list_single_node_add(
        pl : *const JkPropertyList,
        val : *const PropertyValue,
        ) -> *const PropertyValue;

    fn property_list_option_add(
        ps : *const JkPropertyList,
        name : *const c_char,
        value : *const c_char
        ) -> *const PropertyValue;

    fn property_list_vec_add(
        pl : *const JkPropertyList,
        name : *const c_char,
        len : c_int
        ) -> *const PropertyValue;

    pub fn property_list_cb_get(pl : *const JkPropertyList) -> *const ui::JkPropertyCb;

    pub fn jk_property_list_register_cb(
        property : *const JkPropertyList,
        data : *const PropertyList,
        panel_move : ui::PanelGeomFunc
        );

    pub fn jk_property_list_register_vec_cb(
        property : *const JkPropertyList,
        vec_add : RegisterChangeFunc,
        vec_del : RegisterChangeFunc);

    fn property_list_vec_update(
        pv : *const PropertyValue,
        len : c_int);

    pub fn property_expand(
        pv : *const PropertyValue);

    pub fn property_list_enum_update(
        pv : *const ui::PropertyValue,
        value : *const c_char);

    fn property_show(obj : *const JkPropertyList, b : bool);
}

pub struct PropertyList
{
    pub name : String,
    pub jk_property_list : *const JkPropertyList,
    pub pv : RefCell<HashMap<String, *const PropertyValue>>,
    visible : Cell<bool>,
    pub id : uuid::Uuid,
    pub config : PropertyConfig,
}

impl PropertyList
{
    pub fn new(
        window : *const Window,
        pc : &PropertyConfig
        ) -> PropertyList
    {
        PropertyList {
            name : String::from("property_name"),
            jk_property_list : unsafe {jk_property_list_new(
                    window,
                    pc.x, pc.y, pc.w, pc.h)},
            pv : RefCell::new(HashMap::new()),
            visible: Cell::new(true),
            id : uuid::Uuid::new_v4(),
            config : pc.clone(),
        }
    }

    /*
    pub fn set_object(&mut self, o : &object::Object)
    {
        unsafe { property_list_clear(self.jk_property_list); }
        self.pv.clear();

        unsafe {
            property_list_group_add(
                self.jk_property_list,
                CString::new("object".as_bytes()).unwrap().as_ptr());
        }
        //let mut v = Vec::new();
        //v.push("object".to_owned());
        o.create_widget(self, "object", 1, false);

        self.add_tools();
    }
    */

    fn _set_prop(&self, p : &PropertyShow, title : &str)
    {
        unsafe { property_list_clear(self.jk_property_list); }
        self.pv.borrow_mut().clear();

        unsafe {
            property_list_group_add(
                self.jk_property_list,
                CString::new(title.as_bytes()).unwrap().as_ptr());
        }
        //TODO replace ""
        //p.create_widget(self, "", 1, false);

        self.add_tools();
    }


    /*
    pub fn set_scene(&mut self, s : &scene::Scene)
    {
        unsafe { property_list_clear(self.jk_property_list); }
        self.pv.clear();

        unsafe {
            property_list_group_add(
                self.jk_property_list,
                CString::new("scene".as_bytes()).unwrap().as_ptr());
        }
        //let mut v = Vec::new();
        //v.push("object".to_owned());
        s.create_widget(self, "scene", 1, false);
    }
    */


    fn add_tools(&self)
    {
        //add component
        // add as prefab
        // if linked to prefab :
        // State : linked, inherit
        // operation : change state : if linked, remove link(set independant)
        //TODO
        unsafe {
            property_list_group_add(
                self.jk_property_list,
                CString::new("tools").unwrap().as_ptr());
        }
    }

    pub fn data_set(&self, data : *const c_void)
    {
        //TODO
        //unsafe { property_data_set(self.jk_property, data); }
    }

    pub fn update_object_property(&self, object : &PropertyShow, prop : &str)
    {
        // update_widget might add/remove/update self.pv so we have to copy it
        // and check
        let copy = self.pv.borrow().clone();

        println!("UPDATE OBJECT PROP '{}'", prop);

        for (f,pv) in &copy {
            match self.pv.borrow().get(f) {
                Some(p) => if *p != *pv {
                    panic!("different pointer???");
                    continue
                },
                None => continue
            }

            if f.starts_with(prop) {
                let yep = ui::make_vec_from_str(f);
                //if let Some(ppp) = find_property_show(object, yep.clone()) {
                    //ppp.update_widget(*pv);
                //}
                //let test = |ps| {};
                object.update_property(self, prop, yep);
                //object.callclosure(&test);
            }
        }
    }

    pub fn update_object(&self, object : &PropertyShow, but : &str)
    {
        // update_widget might add/remove/update self.pv so we have to copy it
        // and check
        let copy = self.pv.borrow().clone();
        for (f,pv) in &copy {
            match self.pv.borrow().get(f) {
                Some(p) => if *p != *pv {
                    panic!("different pointer???222");
                    continue
                },
                None => continue
            }
            let fstr : &str = f.as_ref();
            //if f.as_ref() as &str == but {
            if fstr == but {
                println!("buuuuuuuuuuuuuuuuuuuuuuuuuut: {} ", f);
                continue;
            }
            let yep = ui::make_vec_from_str(f);
            match ui::find_property_show(object, yep.clone()) {
                Some(ppp) => {
                    ppp.update_widget(*pv);
                },
                None => {
                    println!("could not find prop : {:?}", yep);
                }
            }
        }
    }

    pub fn set_visible(&self, b : bool)
    {
        self.visible.set(b);
        unsafe {
            property_show(self.jk_property_list, b);
        }
    }

    pub fn visible(&self) -> bool
    {
        self.visible.get()
    }

}

impl<S:SceneT> ui::Widget<S> for PropertyList
{
    fn get_id(&self) -> Uuid
    {
        self.id
    }

    fn handle_change_prop(&self, prop_user : &PropertyShow, name : &str)
    {
        self.update_object_property(prop_user, name);
    }
}


pub extern fn contract(
    widget_cb_data: *const c_void,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    /*
    let (p,_) = get_widget_data(widget_cb_data);

    unsafe {
        property_list_nodes_remove(
            p.jk_property_list,
            data as *const c_char
            );
    };

    let datachar = data as *const i8;
    let s = unsafe {CStr::from_ptr(datachar).to_bytes()};
    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            println!("problem with the path");
            return;}
    };

    p.config.expand.remove(path);

    let clone = p.pv.borrow().clone();

    for (key,pv) in &clone {
        let starts_with_path = {
            let ks : &str = key.as_ref();
            ks.starts_with(path) && ks != path
        };

        //if key.as_ref().starts_with(path) && key.as_ref() != path  {
        if starts_with_path {
            match p.pv.borrow_mut().remove(key) {
                Some(_) => println!("yes I removed {}", key),
                None => println!("could not find {}", key)
            }
        }
    }
    */
}

/*
fn get_widget_data<'a>(widget_data : *const c_void) ->
    //(&'a mut ui::PropertyList, &'a mut Box<ui::WidgetContainer>)
    (&'a mut ui::PropertyList, &'a mut ui::WidgetContainer)
{
    println!("GET WIDGET DATAAAAAAAAAAAAAAA, this is old so crash, use get_widget_data2");

    let wcb : & ui::WidgetCbData = unsafe {mem::transmute(widget_data)};
    //let p : &mut ui::PropertyList = unsafe {mem::transmute(wcb.widget)};
    let p : &mut Box<ui::PropertyList> = unsafe {mem::transmute(wcb.widget)};
    //let container : &mut Box<ui::WidgetContainer> = unsafe {mem::transmute(wcb.container)};
    let container : &mut ui::WidgetContainer = &mut *wcb.container.write().unwrap();

    //(p, container)
    (&mut **p, container)
}
*/

fn get_widget_data2<'a, S:SceneT+'static>(
    widget_data : *const c_void, //ui::WidgetCbData<S>
    ) ->
    (Rc<ui::PropertyWidgetGen<S>>, Arw<ui::WidgetContainer<S>>)
{
    let wcb : &ui::WidgetCbData<S> = unsafe { &*(widget_data as *const ui::WidgetCbData<S>) };
    let container = wcb.container.read().unwrap();
    let property = container.property.widget.as_ref().unwrap().clone();
    (property, wcb.container.clone())
}


fn changed_set<T : Any+Clone+PartialEq, S:SceneT>(
    widget_data : *const c_void,//ui::WidgetCbData<S>,
    property : *const c_void,
    old : Option<&T>,
    new : &T,
    action : i32
    )
{
    //let s : &String = unsafe {mem::transmute(property)};
    //println!("SSSSSSSSSSSSSSS : {}", s);

    //return;
    let node : &Weak<RefCell<ui::PropertyNode>> = unsafe {mem::transmute(property)};
    let node = if let Some(n) = node.upgrade() {
        println!("node is : {} ", n.borrow().name);
        n
    }
    else {
        panic!("problem with node");
    };

    let path = &node.borrow().get_path();

    println!("changed_set : {}", path);

    let (p, container) = get_widget_data2::<S>(widget_data);
    let container = &mut *container.write().unwrap();

    let change = match (old, action) {
        (Some(oldd), 1) => {
            if let Some(id) = p.get_current_id() {
                container.state.request_operation_property_old_new(
                    id,
                    path,
                    box oldd.clone(),
                    box new.clone(),
                    &mut *container.data
                    )
            }
            else
            {
                println!("property widget doesn't seem to have a property set to it");
                operation::Change::None
                /*
                container.request_operation_old_new(
                    path,
                    box oldd.clone(),
                    box new.clone())
                    */
            }
        },
        _ => {
            if let Some(pid) = p.get_current_id() {
                //if let Some(mut ppp) = container.data.get_property_user_copy(pid) {
                if let Some((mut ppp, newpath)) = container.data.get_property_write_copy(pid, path) {
                    container.state.request_direct_change_property(&mut *ppp, &newpath,new)
                }
                else {
                    operation::Change::None
                }
            }
            //container.request_direct_change(path, new)
            else {
                operation::Change::None
            }
        }
    };

    //container.handle_change(&change, p.id);
    container.handle_change(&change, p.get_id());
}

fn changed_enum<T : Any+Clone+PartialEq, S:SceneT>(
    widget_data : *const c_void,//ui::WidgetCbData<S>,
    property : *const c_void,
    old : Option<&T>,
    new : &T,
    )
{
    let node : &Weak<RefCell<ui::PropertyNode>> = unsafe {mem::transmute(property)};
    let node = if let Some(n) = node.upgrade() {
        n
    }
    else {
        panic!("problem with node");
    };

    let path = &node.borrow().get_path();

    let (p, container) = get_widget_data2::<S>(widget_data);
    let container = &mut *container.write().unwrap();

    let change = {
        /*
        container.request_operation_old_new_enum(
            path,
            box new.clone())
            */

        if let Some(id) = p.get_current_id() {
            if let Some(oldie) = old {
                container.state.request_operation_property_old_new_dontcheckequal(
                    id,
                    path,
                    box oldie.clone(),
                    box new.clone(),
                    &mut *container.data
                    )
            }
            else {
                operation::Change::None
            }
        }
        else
        {
            println!("property widget doesn't seem to have a property set to it");
            operation::Change::None
        }

    };

    container.handle_change(&change, p.get_id());
}

fn changed_option<S:SceneT>(
    widget_cb_data : *const c_void, //ui::WidgetCbData<S>,
    property : *const c_void,
    old : &str,
    new : &str
    )
{
    panic!("TODO");
    /*
    let node : Weak<RefCell<ui::PropertyNode>> = unsafe {mem::transmute(property)};
    let node = if let Some(n) = node.upgrade() {
        n
    }
    else {
        panic!("problem with node");
    };

    let path = &node.borrow().get_path();

    let (p, container) = get_widget_data(widget_cb_data);

    let change = if let Some(ref cur) = *p.current.borrow() {

        if new == "Some" {
            container.request_operation_option_to_some(cur.clone(), path)
        }
        else {

            let option = match *cur {
                RefMut::Arc(ref a) => a.read().unwrap().get_property_hier(path),
                RefMut::Cell(ref c) => c.borrow().get_property_hier(path)
            };

            if let Some(old) = option {
                container.request_operation_option_to_none(
                    (*cur).clone(),
                    path,
                    old)
            }
            else {
                operation::Change::None
            }
            //container.request_operation_option_to_none(path)
        }
    }
    else {
        operation::Change::None
    };

    container.handle_change(&change, p.id);
    */
}

pub extern fn expand(
    widget_cb_data: *const c_void,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    /*
    let datachar = data as *const i8;
    let s = unsafe {CStr::from_ptr(datachar).to_bytes()};

    let (p, container) = get_widget_data(widget_cb_data);

    let path = match str::from_utf8(s) {
        Ok(pp) => pp,
        _ => {
            panic!("problem with the path");
        }
    };

    let vs = ui::make_vec_from_str(&path.to_owned());

    //TODO factorize this and others
    println!("factorize this and others, the path is : {:?}", vs);
    if let Some(ref cur) = *p.current.borrow() {
        match *cur {
            RefMut::Arc(ref a) =>
            {
                //a.read().unwrap().find_and_create(p, vs.clone(), 0);

            },
            RefMut::Cell(ref c) =>
            {
                //c.borrow().find_and_create(p, vs.clone(), 0);
            }
        }

        p.config.expand.insert(path.to_owned());
    }
    else {
        println!("no current prop....... {}", path);
    }
    */

}


pub extern fn changed_set_float<S:SceneT>(
//pub extern fn changed_set_float(
    app_data : *const c_void, //ui::WidgetCbData<S>,
    property : *const c_void,
    data : *const c_void) {

    println!("changed_set_float : {:?}", property);

    let f : & f64 = unsafe {mem::transmute(data)};
    changed_set::<f64,S>(app_data, property, None, f, 0);
}

pub extern fn changed_set_string<S:SceneT>(
    app_data : *const c_void, //ui::WidgetCbData<S>,
    property : *const c_void,
    data : *const c_void) {

    let datachar = data as *const i8;
    let s = unsafe {CStr::from_ptr(datachar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_owned(),
        _ => {
            return;
        }
    };
    changed_set::<String,S>(app_data, property, None, &ss, 0);
}

pub extern fn changed_set_enum<S:SceneT>(
    app_data : *const c_void, //ui::WidgetCbData<S>,
    property : *const c_void,
    data : *const c_void) {
    println!("DOES NOT NO ANYTHING");
}

pub extern fn register_change_string<S:SceneT>(
    app_data : *const c_void, //ui::WidgetCbData<S>,
    property : *const c_void,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let newchar = new as *const i8;
    let s = unsafe {CStr::from_ptr(newchar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_owned(),
        _ => {
            println!("error");
            return;
        }
    };

    if action == 1 && old != ptr::null() {
        let oldchar = old as *const i8;
        let so = unsafe {CStr::from_ptr(oldchar).to_bytes()};
        let sso = match str::from_utf8(so) {
            Ok(ssso) => ssso.to_owned(),
            _ => {
                println!("error");
                return;
            }
        };

        changed_set::<String,S>(app_data, property, Some(&sso), &ss, action);
    }
    else {
        changed_set::<String,S>(app_data, property, None, &ss, action);
    }
}

pub extern fn register_change_float<S:SceneT>(
    app_data : *const c_void, //ui::WidgetCbData<S>,
    property : *const c_void,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let fnew : & f64 = unsafe {mem::transmute(new)};

    if action == 1 && old != ptr::null() {
        let fold : & f64 = unsafe {mem::transmute(old)};
        changed_set::<f64,S>(app_data, property, Some(fold), fnew, action);
    }
    else {
        changed_set::<f64,S>(app_data, property, None, fnew, action);
    }
}

pub extern fn register_change_enum<S:SceneT>(
    widget_cb_data : *const c_void, //ui::WidgetCbData<S>,
    property : *const c_void,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    )
{
    let newchar = new as *const i8;
    let s = unsafe {CStr::from_ptr(newchar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_owned(),
        _ => {
            println!("error");
            return
        }
    };

    //println!("the string is {}", ss);
    if action == 1 && old != ptr::null() {
        let oldchar = old as *const i8;
        let so = unsafe {CStr::from_ptr(oldchar).to_bytes()};
        let sso = match str::from_utf8(so) {
            Ok(ssso) => ssso.to_owned(),
            _ => {
                println!("error");
                return
            }
        };
        changed_enum::<String,S>(widget_cb_data, property, Some(&sso), &ss);
    }
    else {
        changed_enum::<String,S>(widget_cb_data, property, None, &ss);
    }
}

pub extern fn register_change_option<S:SceneT>(
    widget_cb_data : *const c_void, //ui::WidgetCbData<S>,
    property : *const c_void,
    old : *const c_void,
    new : *const c_void,
    action : c_int
    ) {

    let newchar = new as *const i8;
    let s = unsafe {CStr::from_ptr(newchar).to_bytes()};
    let ss = match str::from_utf8(s) {
        Ok(sss) => sss.to_owned(),
        _ => {
            println!("error");
            return
        }
    };

    //println!("the string is {}", ss);
    if old == ptr::null() {
        println!("option : old is null, return");
        return;
    }

    let oldchar = old as *const i8;
    let so = unsafe {CStr::from_ptr(oldchar).to_bytes()};
    let sso = match str::from_utf8(so) {
        Ok(ssso) => ssso.to_owned(),
        _ => {
            println!("error");
            return
        }
    };

    changed_option::<S>(widget_cb_data, property, &sso, &ss);
}

fn get_str<'a>(cstr : *const c_char) -> Option<&'a str>
{
    let s = unsafe {CStr::from_ptr(cstr).to_bytes()};
    str::from_utf8(s).ok()
}

impl PropertyWidget for PropertyList
{
    fn add_simple_item(&self, field : &str, item : *const PropertyValue)
    {
        unsafe {
            property_list_single_item_add(
                self.jk_property_list,
                item);
        }

        self.pv.borrow_mut().insert(field.to_owned(), item);
    }

    fn add_option(&self, field : &str, is_some : bool) -> *const PropertyValue
    {
        let f = CString::new(field.as_bytes()).unwrap();
        let type_value = match is_some {
            true => "Some",
            false => "None"
        };

        let v = CString::new(type_value.as_bytes()).unwrap();

        unsafe {
            let pv = property_list_option_add(
                self.jk_property_list,
                f.as_ptr(),
                v.as_ptr());

            if pv != ptr::null() {
                self.pv.borrow_mut().insert(field.to_owned(), pv);
            }

            pv
        }
    }

    fn add_vec(&self, name : &str, len : usize)
    {
        let f = CString::new(name.as_bytes()).unwrap();
        let pv = unsafe {
            property_list_vec_add(
                self.jk_property_list,
                f.as_ptr(),
                len as c_int
                )
        };

        if pv != ptr::null() {
            self.pv.borrow_mut().insert(name.to_owned(), pv);
        }

        if self.config.expand.contains(name) {
            unsafe {
                property_expand(pv);
            }
        }
    }

    fn add_vec_item(&self, field : &str, widget_entry : *const PropertyValue, index : usize)
    {
        /*
        I put this in comment because I remove is_node. I also added index.
        unsafe {
            let pv = property_list_single_vec_add(
                self.jk_property_list,
                widget_entry,
                is_node
                );

            if pv != ptr::null() {
                self.pv.borrow_mut().insert(field.to_owned(), widget_entry);
            }

            if self.config.expand.contains(field) {
                property_expand(widget_entry);
            }
        }
        */
    }

    fn del_vec_item(&self, field : &str, index : usize)
    {
        //TODO TODOTODO
    }


    /*
    fn update_option(&mut self, widget_entry : *const PropertyValue, is_some : bool)
    {
        let s = match is_some {
            true => "Some",
            false => "None"
        };

        let v = CString::new(s.as_bytes()).unwrap();
        unsafe {
            ui::property::property_list_option_update(
                widget_entry,
                v.as_ptr());
        };
    }
    */

    /*
    fn update_vec(&mut self, widget_entry : *const PropertyValue, len : usize)
    {
        unsafe { property_list_vec_update(widget_entry, len as c_int); }
        unsafe { property_expand(widget_entry); }
    }
    */

    fn update_enum(&self, path : &str, widget_entry : *const PropertyValue, value : &str)
    {
        let v = CString::new(value.as_bytes()).unwrap();
        unsafe {
            property_list_enum_update(widget_entry, v.as_ptr());
        }
    }

    fn update_vec(&self, widget_entry : *const PropertyValue, len : usize)
    {
        unsafe { property_list_vec_update(widget_entry, len as c_int); }
        unsafe { property_expand(widget_entry); }
    }

}

impl<Scene:SceneT> PropertyWidgetGen<Scene> for PropertyList
{
    fn get_current_id(&self) -> Option<Scene::Id>
    {
        println!("TODO {}, {}", file!(), line!());
        None
    }

    fn set_current_id(&self, p : &PropertyShow, id : Scene::Id, title : &str)
    {
        println!("TODO {}, {}", file!(), line!());
    }
}
