use std::collections::HashMap;
use libc::{c_char, c_void, c_int};
use std::collections::{LinkedList};//,Deque};
use std::ptr;
use std::ffi::CString;
use uuid::Uuid;

use ui::Window;
use ui::{RefMut,PropertyUser};
use ui;
use data::{ToId, SceneT};

#[repr(C)]
pub struct Elm_Object_Item;
#[repr(C)]
pub struct JkTree;

#[link(name = "joker")]
extern {
    fn window_tree_new(
        window : *const Window,
        x : c_int,
        y : c_int,
        w : c_int,
        h : c_int
        ) -> *const JkTree;
    pub fn tree_register_cb(
        tree : *const JkTree,
        data : *const c_void,
        name_get : extern fn(data : *const c_void) -> *const c_char,
        selected : extern fn(data : *const c_void) -> (),
        can_expand : extern fn(data : *const c_void) -> bool,
        expand : extern fn(tree: *const c_void, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        //sel : extern fn(tree: *const TreeSelectData, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        //unsel : extern fn(tree: *const TreeSelectData, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        sel : extern fn(tree: *const ui::WidgetCbData, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        unsel : extern fn(tree: *const ui::WidgetCbData, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        panel_move : ui::PanelGeomFunc
        );

    fn tree_object_add(
        tree : *const JkTree,
        object : *const c_void,
        parent : *const Elm_Object_Item,
        ) -> *const Elm_Object_Item;

    fn tree_object_remove(
        item : *const Elm_Object_Item);

    fn tree_item_select(item : *const Elm_Object_Item);
    fn tree_item_update(item : *const Elm_Object_Item);
    fn tree_item_expand(item : *const Elm_Object_Item);
    fn tree_deselect_all(item : *const JkTree);
    fn tree_update(tree : *const JkTree);
    fn tree_show(obj : *const JkTree, b : bool);
    fn tree_clear(obj : *const JkTree);
}

struct ItemData
{
    object : ui::def::Object,
    name : String,
}

impl ItemData
{
    fn new(o : ui::def::Object, name : String) -> ItemData
    {
        ItemData {
            object : o,
            name : name
        }
    }
}

pub struct Tree
{
    pub name : String,
    //TODO change the key
    //objects : HashMap<Arc<RwLock<object::Object>>, *const Elm_Object_Item >
    //objects : HashMap<String, *const Elm_Object_Item>,
    objects : HashMap<ui::def::Id, *const Elm_Object_Item>,
    pub jk_tree : *const JkTree,
    pub id : Uuid,
    pub config : ui::WidgetConfig,
    scene : Option<ui::def::Id>
}

impl Tree
{
    pub fn new(
        window : *const Window,
        config : &ui::WidgetConfig
        ) -> Tree // Box<Tree>
    {
        //let mut t = box Tree {
        let mut t = Tree {
            name : String::from("tree_name"),
            objects : HashMap::new(),
            jk_tree : unsafe {window_tree_new(
                    window, config.x, config.y, config.w, config.h)},
            id : Uuid::new_v4(),
            config : config.clone(),
            scene : None
        };

        t.set_visible(config.visible);

        t
    }

    pub fn set_scene(&mut self, scene : &ui::def::Scene)
    {
        unsafe {tree_clear(self.jk_tree);}
        self.objects.clear();

        for o in scene.get_objects() {
            let parent_id = scene.get_parent(o.clone()).map(|x| x.to_id());
            let name = scene.get_object_name(o.clone());
            self._add_object(scene.get_parent(o.clone()).map(|x| x.to_id()), o, name);
        }

        self.scene = Some(scene.to_id());
    }

    fn _add_object(&mut self, parent : Option<ui::def::Id>, o : &ui::def::Object, name : String)
    {
        let eoi = match parent {
            Some(ref p) =>  {
                match self.objects.get(p) {
                    Some(item) => {
                        unsafe { 
                            tree_object_add(
                                self.jk_tree,
                                Box::into_raw(box ItemData::new(o.clone(), name)) as *const c_void,
                                *item)
                        }
                    },
                    None => {
                        println!("problem with tree, could not find parent item");
                        ptr::null()
                    }
                }

            },
            None => {
                unsafe {
                    tree_object_add(
                        self.jk_tree,
                        Box::into_raw(box ItemData::new(o.clone(), name)) as *const c_void,
                        ptr::null())
                }
            }
        };

        if eoi != ptr::null() {
            self.objects.insert(o.to_id(), eoi);
        }

    }

    pub fn add_object(
        &mut self,
        parent : Option<ui::def::Id>,
        object : ui::def::Object,
        name : String
        )
    {
        if self.objects.contains_key(&object.to_id()) {
            return;
        }

        self._add_object(parent, &object, name);
    }

    pub fn add_objects(
        &mut self,
        parents : &[Option<ui::def::Id>],
        objects : &[ui::def::Object],
        names : Vec<String>,
        )
    {
        for ((o,p),n) in objects.iter().zip(parents.iter()).zip(names.into_iter()) {
            self.add_object(*p, o.clone(), n);
        }
    }

    pub fn remove_objects_by_id(&mut self, ids : Vec<ui::def::Id>)
    {
        for id in &ids {
            let item = self.objects.remove(id);
            if let Some(i) = item {
                unsafe { tree_object_remove(i);}
            }
        }
    }

    pub fn select(&mut self, id: &ui::def::Id)
    {
        unsafe { tree_deselect_all(self.jk_tree); }
        self._select(id);
    }

    pub fn select_objects(&mut self, ids: Vec<ui::def::Id>)
    {
        unsafe { tree_deselect_all(self.jk_tree); }
        for id in &ids {
            self._select(id);
        }
    }

    fn _select(&mut self, id: &ui::def::Id)
    {
        if let Some(item) = self.objects.get(id) {
            unsafe {tree_item_select(*item);}
        }
    }


    pub fn set_selected(&mut self, ids: LinkedList<ui::def::Id>)
    {
        unsafe { tree_deselect_all(self.jk_tree); }

        for id in &ids {
            if let Some(item) = self.objects.get(id) {
                unsafe {tree_item_select(*item);}
            }
        }
    }

    pub fn update(&self)
    {
        unsafe { tree_update(self.jk_tree); }
    }

    pub fn update_object(& self, id: &ui::def::Id)
    {
        if let Some(item) = self.objects.get(id) {
            unsafe {tree_item_update(*item);}
        }
    }

    pub fn set_visible(&mut self, b : bool)
    {
        self.config.visible = b;
        unsafe {
            tree_show(self.jk_tree, b);
        }
    }

    pub fn visible(&self) -> bool
    {
        self.config.visible
    }

    pub fn get_config(&self) -> ui::WidgetConfig
    {
        self.config.clone()
    }
}

pub extern fn name_get(data : *const c_void) -> *const c_char
{
    let item_data : &ItemData = unsafe {&* (data as *const ItemData)};

    //println!("name get {:?}", o);
    let cs = CString::new(item_data.name.as_bytes()).unwrap();
    cs.as_ptr()
}

pub extern fn item_selected(data : *const c_void) -> ()
{
    let item_data : &ItemData = unsafe {&* (data as *const ItemData)};
    println!("item_selected callback ! {}, but this function does nothing for now ", item_data.name);
}

pub extern fn can_expand(data : *const c_void) -> bool
{
    let item_data : &ItemData = unsafe {&* (data as *const ItemData)};

    //println!("can expand :{}", o.read().unwrap().children.is_empty());
    //return !o.read().unwrap().children.is_empty();
    
    println!("TODO can_expand, {}, {}", file!(), line!());
    false
}

pub extern fn expand(
    widget_cb_data: *const c_void,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let item_data : &ItemData = unsafe {&* (data as *const ItemData)};

    let wcb : & ui::WidgetCbData = unsafe {&*(widget_cb_data as *const ui::WidgetCbData)};
    let container = &mut *wcb.container.write().unwrap();
    let t : &mut Tree = &mut *container.tree.as_mut().unwrap();
    use uuid;
    use data::DataT;

    let scene : &ui::def::Scene = if let Some(s_id) = t.scene {
        if let Some(s) = container.data.get_scene(s_id) {
            s
        }
        else {
            return;
        }
    }
    else {
        return;
    };

    println!("expanding ! {} ", item_data.name);
    println!("expanding ! tree name {} ", t.name);

    //TODO
    for c in scene.get_children(item_data.object.clone()) {
        //println!("expanding ! with child {} ", (*c).read().unwrap().name);
        unsafe {
            let cid = c.to_id();
            let name = scene.get_object_name(c.clone());
            let eoi = tree_object_add(
                t.jk_tree,
                Box::into_raw(box ItemData::new(c.clone(), name)) as *const c_void,
                parent);
            t.objects.insert(cid, eoi);
        }
    }
}

pub extern fn selected(
    widget_cb_data: *const ui::WidgetCbData,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let wcb : & ui::WidgetCbData = unsafe {&*(widget_cb_data as *const ui::WidgetCbData)};
    let container = &mut *wcb.container.write().unwrap();
    let tree_id = container.tree.as_ref().unwrap().id;

    let item_data : &ItemData = unsafe {&* (data as *const ItemData)};

    println!("selected callback, TODO do the following in widget container 'handle' ");

    container.handle_event(ui::Event::SelectObject(item_data.object.clone()), tree_id);
}

pub extern fn unselected(
    widget_cb_data: *const ui::WidgetCbData,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let wcb : & ui::WidgetCbData = unsafe {&*(widget_cb_data as *const ui::WidgetCbData)};
    let container = &mut *wcb.container.write().unwrap();
    let tree_id = container.tree.as_ref().unwrap().id;

    let item_data : &ItemData = unsafe {&* (data as *const ItemData)};

    println!("TODO,unselect do the following in widget container 'handle'");
    container.handle_event(ui::Event::UnselectObject(item_data.object.clone()), tree_id);
}

impl ui::Widget for Tree
{
    fn get_id(&self) -> Uuid
    {
        self.id
    }

    fn handle_change_prop(&self, prop_user : &PropertyUser, name : &str)
    {
        if name == "name" {
            self.update_object(&prop_user.get_id());
        }
    }
}

pub extern fn panel_move(
    widget_cb_data : *const c_void,
    x : c_int, y : c_int, w : c_int, h : c_int)
{
    println!("panel geom !!!!!!!!! {}, {}, {}, {}", x, y, w, h);
    let wcb : & ui::WidgetCbData = unsafe {&*(widget_cb_data as *const ui::WidgetCbData)};
    let container = &mut *wcb.container.write().unwrap();
    let t : &mut Tree = &mut *container.tree.as_mut().unwrap();

    t.config.x = x;
    t.config.y = y;
    t.config.w = w;
    t.config.h = h;
}

