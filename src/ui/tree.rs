use std::collections::HashMap;
use libc::{c_char, c_void, c_int};
use std::collections::{LinkedList};//,Deque};
use std::ptr;
use std::ffi::CString;
use uuid::Uuid;

use ui::Window;
use ui::{PropertyUser};
use ui;
use data::{ToId, SceneT, Data};

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
        sel : extern fn(tree: *const c_void, data : *const c_void, parent: *const Elm_Object_Item) -> (),
        unsel : extern fn(tree: *const c_void, data : *const c_void, parent: *const Elm_Object_Item) -> (),
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

struct ItemData<Scene : SceneT>
{
    object : Scene::Object,
    name : String,
    has_children : bool
}

impl<Scene:SceneT> ItemData<Scene>
{
    fn new(o : Scene::Object, name : String, has_children : bool) -> ItemData<Scene>
    {
        ItemData {
            object : o,
            name : name,
            has_children : has_children
        }
    }
}

pub struct Tree<Scene : SceneT>
{
    pub name : String,
    //TODO change the key
    //objects : HashMap<Arc<RwLock<object::Object>>, *const Elm_Object_Item >
    //objects : HashMap<String, *const Elm_Object_Item>,
    objects : HashMap<Scene::Id, *const Elm_Object_Item>,
    pub jk_tree : *const JkTree,
    pub id : Uuid,
    pub config : ui::WidgetConfig,
    scene : Option<Scene::Id>
}

impl<Scene: SceneT> Tree<Scene>
{
    pub fn new(
        window : *const Window,
        config : &ui::WidgetConfig
        ) -> Tree<Scene>
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

    pub fn set_scene(&mut self, scene : &Scene)
    {
        unsafe {tree_clear(self.jk_tree);}
        self.objects.clear();

        for o in scene.get_objects() {
            let parent_id = scene.get_parent(o.clone()).map(|x| x.to_id());
            let name = scene.get_object_name(o.clone());
            let has_children = !scene.get_children(o.clone()).is_empty();
            self._add_object(scene.get_parent(o.clone()).map(|x| x.to_id()), o, name, has_children);
        }

        self.scene = Some(scene.to_id());
    }

    fn _add_object(&mut self, parent : Option<Scene::Id>, o : &Scene::Object, name : String, has_children : bool)
    {
        let eoi = match parent {
            Some(ref p) =>  {
                match self.objects.get(p) {
                    Some(item) => {
                        let item_data : ItemData<Scene> = 
                            ItemData::new(o.clone(), name, has_children);
                        unsafe { 
                            tree_object_add(
                                self.jk_tree,
                                Box::into_raw(box item_data) as *const c_void,
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
                let item_data : ItemData<Scene> = 
                    ItemData::new(o.clone(), name, has_children);
                unsafe {
                    tree_object_add(
                        self.jk_tree,
                        Box::into_raw(box item_data) as *const c_void,
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
        parent : Option<Scene::Id>,
        object : Scene::Object,
        name : String,
        has_children : bool
        )
    {
        if self.objects.contains_key(&object.to_id()) {
            return;
        }

        self._add_object(parent, &object, name, has_children);
    }

    pub fn add_objects(
        &mut self,
        parents : &[Option<Scene::Id>],
        objects : &[Scene::Object],
        names : Vec<String>,
        has_children : &[bool],
        )
    {
        for (((o,p),n),child) in objects.iter().zip(parents.iter()).zip(names.into_iter()).zip(has_children.iter()) {
            self.add_object(p.clone(), o.clone(), n, *child);
        }
    }

    pub fn remove_objects_by_id(&mut self, ids : Vec<Scene::Id>)
    {
        for id in &ids {
            let item = self.objects.remove(id);
            if let Some(i) = item {
                unsafe { tree_object_remove(i);}
            }
        }
    }

    pub fn select(&mut self, id: &Scene::Id)
    {
        unsafe { tree_deselect_all(self.jk_tree); }
        self._select(id);
    }

    pub fn select_objects(&mut self, ids: Vec<Scene::Id>)
    {
        unsafe { tree_deselect_all(self.jk_tree); }
        for id in &ids {
            self._select(id);
        }
    }

    fn _select(&mut self, id: &Scene::Id)
    {
        if let Some(item) = self.objects.get(id) {
            unsafe {tree_item_select(*item);}
        }
    }


    pub fn set_selected(&mut self, ids: LinkedList<Scene::Id>)
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

    pub fn update_object(&self, id: &Scene::Id)
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

pub extern fn name_get<Scene:SceneT>(data : *const c_void) -> *const c_char
{
    let item_data : &ItemData<Scene> = unsafe {&* (data as *const ItemData<Scene>)};

    //println!("name get {:?}", o);
    let cs = CString::new(item_data.name.as_bytes()).unwrap();
    cs.as_ptr()
}

pub extern fn item_selected<Scene:SceneT>(data : *const c_void) -> ()
{
    let item_data : &ItemData<Scene> = unsafe {&* (data as *const ItemData<Scene>)};
    println!("item_selected callback ! {}, but this function does nothing for now ", item_data.name);
}

pub extern fn can_expand<Scene:SceneT>(data : *const c_void) -> bool
{
    let item_data : &ItemData<Scene> = unsafe {&* (data as *const ItemData<Scene>)};

    println!("TODO can_expand, {}, {}", file!(), line!());
    item_data.has_children
}

pub extern fn expand<Scene : SceneT>(
    widget_cb_data: *const c_void,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let item_data : &ItemData<Scene> = unsafe {&* (data as *const ItemData<Scene>)};

    let wcb : & ui::WidgetCbData<Scene> = unsafe {&*(widget_cb_data as *const ui::WidgetCbData<Scene>)};
    let container = &mut *wcb.container.write().unwrap();
    println!("TODO chris uncomment this");
    /*
    let t : &mut Box<Tree<Scene>> = container.tree.as_mut().unwrap();

    let scene : &Scene = if let Some(s_id) = t.scene {
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
            let has_children = !scene.get_children(c.clone()).is_empty();
            let eoi = tree_object_add(
                t.jk_tree,
                Box::into_raw(box ItemData::new(c.clone(), name, has_children)) as *const c_void,
                parent);
            t.objects.insert(cid, eoi);
        }
    }
    */
}

pub extern fn selected<Scene:SceneT>(
    widget_cb_data: *const c_void, //ui::WidgetCbData<Scene>,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let wcb : & ui::WidgetCbData<Scene> = unsafe {&*(widget_cb_data as *const ui::WidgetCbData<Scene>)};
    let container = &mut *wcb.container.write().unwrap();
    let tree_id = container.tree.as_ref().unwrap().id;

    let item_data : &ItemData<Scene> = unsafe {&* (data as *const ItemData<Scene>)};

    println!("selected callback, TODO do the following in widget container 'handle' ");

    println!("TODO chris");
    //container.handle_event(ui::Event::SelectObject(item_data.object.clone()), tree_id);
}

pub extern fn unselected<Scene:SceneT>(
    widget_cb_data: *const c_void,//ui::WidgetCbData<Scene>,
    data : *const c_void,
    parent : *const Elm_Object_Item) -> ()
{
    let wcb : & ui::WidgetCbData<Scene> = unsafe {&*(widget_cb_data as *const ui::WidgetCbData<Scene>)};
    let container = &mut *wcb.container.write().unwrap();
    let tree_id = container.tree.as_ref().unwrap().id;

    let item_data : &ItemData<Scene> = unsafe {&* (data as *const ItemData<Scene>)};

    println!("TODO,unselect do the following in widget container 'handle'");
    println!("TODO chris uncomment");
    //container.handle_event(ui::Event::UnselectObject(item_data.object.clone()), tree_id);
}

impl<Scene:SceneT> ui::Widget<Scene> for Tree<Scene>
{
    fn get_id(&self) -> Uuid
    {
        self.id
    }

    //TODO chris uncomment
    fn handle_change_prop(&self, prop_user : &PropertyUser<Scene>, name : &str)
    {
        if name == "name" {
            println!("TODO chris uncomment");
            //self.update_object(&prop_user.get_id());
        }
    }
}

pub extern fn panel_move<Scene:SceneT>(
    widget_cb_data : *const c_void,
    x : c_int, y : c_int, w : c_int, h : c_int)
{
    println!("panel geom !!!!!!!!! {}, {}, {}, {}", x, y, w, h);
    let wcb : & ui::WidgetCbData<Scene> = unsafe {&*(widget_cb_data as *const ui::WidgetCbData<Scene>)};
    let container = &mut *wcb.container.write().unwrap();
    
    println!("TODO chris uncomment");
    /*
    let t : &mut Tree = &mut *container.tree.as_mut().unwrap();

    t.config.x = x;
    t.config.y = y;
    t.config.w = w;
    t.config.h = h;
    */
}

