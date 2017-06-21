use std::rc::{Rc};
use std::cell::{RefCell};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::fs;

use uuid;

use dormin;
use dormin::{vec, resource, scene, factory, world, object};
use dormin::component::mesh_render;
use dormin::render;
use dormin::property::PropertyGet;
use dormin::input;
use dormin::matrix;
use dormin::transform;

use context;
use util;

static SCENE_SUFFIX: &str = ".scene";
//static WORLD_SUFFIX: &str = ".world";

pub type DataOld = Data<Rc<RefCell<scene::Scene>>>;

pub struct Data<S:SceneT>
{
    pub factory : factory::Factory,
    pub scenes : HashMap<String, S>,

    pub worlds : HashMap<String, Box<dormin::world::World>>,
}

use operation;
use dormin::property::PropertyWrite;

/*
impl<S:SceneT> operation::OperationReceiver for Data<S> {
    type Id = S::Id;
    fn getP(&mut self, id : Self::Id) -> Option<&mut PropertyWrite>
    {
        None
    }

    //fn copy_object(
}
*/

impl operation::OperationReceiver for Data<Rc<RefCell<scene::Scene>>> {
    type Scene = Rc<RefCell<scene::Scene>>;
    fn getP_copy(&mut self, id : <Self::Scene as SceneT>::Id) -> Option<Box<PropertyWrite>>
    {
        for s in self.scenes.values() {
            for o in &s.borrow().objects {
                if o.to_id() == id {
                    return Some(box o.clone());
                }
            }

        }

        None
    }

    fn add_objects(
        &mut self,
        scene_id : <Self::Scene as SceneT>::Id,
        parents : &[Option<<Self::Scene as SceneT>::Id>],
        objects : &[<Self::Scene as SceneT>::Object])
    {
        if let Some(s) = self.get_scene(scene_id) {
            s.borrow_mut().add_objects(parents, objects);
        }
    }


    fn remove_objects(
        &mut self,
        scene_id : <Self::Scene as SceneT>::Id,
        parents : &[Option<<Self::Scene as SceneT>::Id>],
        objects : &[<Self::Scene as SceneT>::Object])
    {
        if let Some(s) = self.get_scene(scene_id) {
            s.remove_objects(parents, objects);
        }
    }

    fn set_camera(&mut self, scene_id : <Self::Scene as SceneT>::Id,
                  camera : Option<<Self::Scene as SceneT>::Object>)
    {
        if let Some(s) = self.get_scene(scene_id) {
            s.set_camera(camera);
        }
    }

    //fn copy_object(
}

impl operation::OperationReceiver for Data<dormin::world::World> {
    type Scene = dormin::world::World;
}


pub trait SceneT : ToId<<Self as SceneT>::Id> {
    type Id : Default + Eq + Clone;
    type Object : ToId<Self::Id> + Clone + world::GetWorld<Self::Object> + GetComponent + PropertyGet;
    type SceneRef : ToId<Self::Id> + Clone;
    fn init_for_play(&mut self, resource : &resource::ResourceGroup);
    fn update(&mut self, dt : f64, input : &input::Input, &resource::ResourceGroup);
    fn get_objects(&self) -> &[Self::Object];
    fn get_objects_vec(&self) -> Vec<Self::Object>
    {
        Vec::new()
    }

    fn get_mmr(&self) -> Vec<render::MatrixMeshRender>
    {
        Vec::new()
    }

    fn get_cameras_vec(&self) -> Vec<matrix::Matrix4>
    {
        Vec::new()
    }

    fn get_camera_obj(&self) -> Option<Self::Object>
    {
        println!("TODO, {}, {}", file!(), line!());
        None
    }

    fn find_objects_by_id(&self, ids : &mut Vec<Self::Id>) -> Vec<Self::Object> {
        Vec::new()
    }

    fn find_object_by_id(&self, id : Self::Id) -> Option<Self::Object> {
        None
    }

    fn get_name(&self) -> String;
    fn set_name(&self, s : String);

    fn save(&self);

    fn add_objects(&self, parents : &[Option<Self::Id>], obs : &[Self::Object])
    {
        println!("TODO, {}, {}", file!(), line!());
    }

    fn remove_objects(&self, parents : &[Option<Self::Id>], obs : &[Self::Object])
    {
        println!("TODO, {}, {}", file!(), line!());
    }

    fn set_camera(&self, ob : Option<Self::Object>)
    {
        println!("TODO, {}, {}", file!(), line!());
    }

    fn get_parent(&self, o : Self::Object) -> Option<Self::Object>
    {
        println!("TODO, {}, {}", file!(), line!());
        None
    }

    fn get_children(&self, o : Self::Object) -> Vec<Self::Object>
    {
        println!("TODO, {}, {}", file!(), line!());
        Vec::new()
    }

    fn set_position(&self, o : Self::Object, v : vec::Vec3)
    {
        println!("TODO, {}, {}", file!(), line!());
    }

    fn set_scale(&self, o : Self::Object, v : vec::Vec3)
    {
        println!("TODO, {}, {}", file!(), line!());
    }

    fn set_orientation(&self, o : Self::Object, ori : transform::Orientation)
    {
        println!("TODO, {}, {}", file!(), line!());
    }

    fn get_position(&self, o : Self::Object) -> vec::Vec3
    {
        println!("TODO, {}, {}", file!(), line!());
        vec::Vec3::default()
    }

    fn get_scale(&self, o : Self::Object) -> vec::Vec3
    {
        println!("TODO, {}, {}", file!(), line!());
        vec::Vec3::default()
    }

    fn get_orientation(&self, o : Self::Object) -> transform::Orientation
    {
        println!("TODO, {}, {}", file!(), line!());
        transform::Orientation::default()
    }

    //TODO use &str instead of string?
    fn get_object_name(&self, o : Self::Object) -> String
    {
        //"TODO name, yo".to_owned()
        format!("TODO {}, {}", file!(), line!())
    }

    fn get_comp_data_value<T:Any+Clone>(&self, o : Self::Object) -> Option<T>
    {
        println!("TODO, {}, {}", file!(), line!());
        None
    }

    fn create_empty_object(&mut self, name : &str) -> Self::Object;

}

impl SceneT for Rc<RefCell<scene::Scene>> {
    type Id = uuid::Uuid;
    type Object = Arc<RwLock<object::Object>>;
    type SceneRef = Rc<RefCell<scene::Scene>>;
    fn update(&mut self, dt : f64, input : &input::Input, res :&resource::ResourceGroup)
    {
        self.borrow_mut().update(dt, input, res);
    }

    fn init_for_play(&mut self, resource : &resource::ResourceGroup)
    {
        self.borrow().init_components(resource);
    }

    fn get_objects(&self) -> &[Self::Object]
    {
        &[]//&self.borrow().objects
    }

    fn get_objects_vec(&self) -> Vec<Self::Object>
    {
        self.borrow().objects.clone()
    }

    fn find_objects_by_id(&self, ids : &mut Vec<Self::Id>) -> Vec<Self::Object> {
        self.borrow().find_objects_by_id(ids)
    }

    fn find_object_by_id(&self, id : Self::Id) -> Option<Self::Object>
    {
        self.borrow().find_object_by_id(&id)
    }

    fn get_name(&self) -> String
    {
        self.borrow().name.clone()
    }

    fn set_name(&self, s : String)
    {
        self.borrow_mut().name = s;
    }

    fn save(&self)
    {
        self.borrow().save();
    }

    fn get_cameras_vec(&self) -> Vec<matrix::Matrix4>
    {
        //self.borrow().cameras.iter().map(|x| x.borrow().object.read().unwrap().get_world_matrix()).collect()
        let mut cams = Vec::new();
        for c in &self.borrow().cameras {
            let cam = c.borrow();
            cams.push(cam.object.read().unwrap().get_world_matrix());
        }

        cams
    }

    fn get_camera_obj(&self) -> Option<Self::Object>
    {
        self.borrow().camera.as_ref().map(|x| x.borrow().object.clone())
    }

    fn get_mmr(&self) -> Vec<render::MatrixMeshRender>
    {
        fn object_to_mmr(o : &object::Object) -> Option<render::MatrixMeshRender>
        {
            o.mesh_render.as_ref().map(|x| render::MatrixMeshRender::new(o.get_world_matrix().clone(), x.clone()))
        }

        fn children_mmr(o : &object::Object) -> Vec<render::MatrixMeshRender>
        {
            o.children.iter().filter_map(|x| object_to_mmr(&*x.read().unwrap())).collect()
        }

        fn object_and_child(o : &object::Object) -> Vec<render::MatrixMeshRender>
        {
            let mut v = children_mmr(o);
            if let Some(m) = object_to_mmr(o) {
                v.push(m);
            }
            v
        }

        let mut v = Vec::new();
        for o in self.borrow().objects.iter() {
            v.append(&mut object_and_child(&*o.read().unwrap()));
        }

        v
    }

    fn add_objects(&self, parents : &[Option<Self::Id>], obs : &[Self::Object])
    {
        self.borrow_mut().add_objects(parents, obs);
    }

    fn remove_objects(&self, parents : &[Option<Self::Id>], obs : &[Self::Object])
    {
        self.borrow_mut().remove_objects(parents, obs);
    }

    fn set_camera(&self, ob : Option<Self::Object>)
    {
        let sc = self.borrow();
        if let Some(ref c) = sc.camera {
            if let Some(o) = ob {
                println!("I set thhe camera !!!!!!!");
                c.borrow_mut().object_id = Some(o.read().unwrap().id.clone());
                c.borrow_mut().object = o;
            }
            else {
                println!("dame 10");
                c.borrow_mut().object_id = None;
            }
        }
    }

    fn get_parent(&self, o : Self::Object) -> Option<Self::Object>
    {
        o.read().unwrap().parent.clone()
    }

    fn get_children(&self, o : Self::Object) -> Vec<Self::Object>
    {
        o.read().unwrap().children.clone()
    }

    fn set_position(&self, o : Self::Object, v : vec::Vec3)
    {
        o.write().unwrap().position = v;
    }

    fn set_scale(&self, o : Self::Object, v : vec::Vec3)
    {
        o.write().unwrap().scale = v;
    }

    fn set_orientation(&self, o : Self::Object, ori : transform::Orientation)
    {
        o.write().unwrap().orientation = ori;
    }

    fn get_position(&self, o : Self::Object) -> vec::Vec3
    {
        o.write().unwrap().position
    }

    fn get_scale(&self, o : Self::Object) -> vec::Vec3
    {
        o.write().unwrap().scale
    }

    fn get_orientation(&self, o : Self::Object) -> transform::Orientation
    {
        o.write().unwrap().orientation
    }

    fn get_object_name(&self, o : Self::Object) -> String
    {
        o.read().unwrap().name.clone()
    }

    fn get_comp_data_value<T:Any+Clone>(&self, o : Self::Object) -> Option<T>
    {
        o.read().unwrap().get_comp_data_value()
    }

    fn create_empty_object(&mut self, name : &str) -> Self::Object
    {
       Arc::new(RwLock::new(object::Object::new(name)))
    }
}

impl ToId<usize> for world::World
{
    fn to_id(&self) -> usize
    {
        println!("TO TODO, world ToId");
        0usize
    }
}

impl ToId<usize> for usize
{
    fn to_id(&self) -> usize
    {
        *self
    }
}

impl SceneT for world::World {
    type Id = usize;
    type Object = usize;
    type SceneRef = usize;
    fn update(&mut self, dt : f64, input : &input::Input, res :&resource::ResourceGroup)
    {
        println!("TODO !!!!!!!!!!!!!!!!!!!!!!");
    }

    fn init_for_play(&mut self, resource : &resource::ResourceGroup)
    {
        println!("TODO !!!!!!!!!!!!!!!!!!!!!! {}, {} ", file!(), line!());
    }

    fn get_objects(&self) -> &[Self::Object]
    {
        //TODO
        &[]
    }

    fn get_name(&self) -> String
    {
        String::from("get_name not implemented")
    }

    fn set_name(&self, s : String)
    {
        println!("TODO !!!!!!!!!!!!!!!!!!!!!! {}, {} ", file!(), line!());
    }

    fn save(&self)
    {
        println!("TODO !!!!!!!!!!!!!!!!!!!!!! {}, {}", file!(), line!());
    }

    fn create_empty_object(&mut self, name : &str) -> Self::Object
    {
        println!("TODO !!!!!!!!!!!!!!!!!!!!!! {}, {}", file!(), line!());
        0usize
    }
}

pub trait DataT<S : SceneT> {
    fn get_scene(&self, id : S::Id) -> Option<&S>;
    fn get_scene_mut(&mut self, id : S::Id) -> Option<&mut S>;
}

impl<S:SceneT> DataT<S> for Data<S>
{
    fn get_scene(&self, id : S::Id) -> Option<&S>
    {
        for v in self.scenes.values() {
            if v.to_id() == id {
                return Some(v)
            }
        }

        None
    }

    fn get_scene_mut(&mut self, id : S::Id) -> Option<&mut S>
    {
        for v in self.scenes.values_mut() {
            if v.to_id() == id {
                return Some(v)
            }
        }

        None
    }

}

impl<S:SceneT> Data<S> {

    pub fn new() -> Data<S> {
        Data {
            factory : factory::Factory::new(),
            scenes : HashMap::new(),

            worlds : HashMap::new(),
        }
    }

    pub fn create_scene_name_with_context(&self, context : &context::Context<S>)
    -> String
    {
        let newname = match context.scene {
            Some(ref sc) => {
                let scene = self.get_scene(sc.to_id()).unwrap();
                let s_name = scene.get_name();
                let old = if s_name.ends_with(SCENE_SUFFIX) {
                    let i = s_name.len() - SCENE_SUFFIX.len();
                    let (yep,_) = s_name.split_at(i);
                    yep
                }
                else {
                    s_name.as_ref()
                };
                String::from(old)
            },
            None => String::from("scene/new.scene")
        };

        create_scene_name(newname)
    }
}

fn create_scene_name(name : String) -> String
{
    let mut i = 0i32;
    let mut s = name.clone();
    loop {
        s.push_str(format!("{:03}",i).as_str());
        s.push_str(SCENE_SUFFIX);

        if let Err(_) = fs::metadata(s.as_str()) {
            break;
        }

        i = i+1;
        s = name.clone();
    }

    s
}

use ui::PropertyUser;
impl Data<Rc<RefCell<scene::Scene>>>
{
    pub fn get_property_user_copy(&self, id : uuid::Uuid) -> Option<Box<PropertyUser>>
    {
        for s in self.scenes.values() {
            if s.to_id() == id {
                return Some(box s.clone());
            }

            for o in &s.borrow().objects {
                if o.to_id() == id {
                    return Some(box o.clone());
                }
            }

        }

        None
    }
}

impl Data<world::World>
{
    pub fn add_empty_scene(&mut self, name : String) -> &mut world::World
    {
        self.scenes.entry(name.clone()).or_insert(
                world::World::new()
            )
    }

    pub fn get_or_load_scene(&mut self, name : &str) -> &mut world::World
    {
        //TODO
        self.scenes.entry(String::from(name)).or_insert(world::World::new())
    }

    pub fn get_or_load_any_scene(&mut self) -> &mut world::World
    {
        //TODO
        self.scenes.entry("todo".to_owned()).or_insert(world::World::new())
    }

    pub fn get_property_user_copy(&self, id : usize) -> Option<Box<PropertyUser>>
    {
        for s in self.scenes.values() {
            if s.to_id() == id {
                //return Some(box s.clone());
            }

            for o in s.get_objects() {
                if o.to_id() == id {
                    //return Some(box o.clone());
                }
            }

        }

        None
    }
}

impl Data<Rc<RefCell<scene::Scene>>>
{
    pub fn add_empty_scene(&mut self, name : String) -> &mut Rc<RefCell<scene::Scene>>
    {
        self.scenes.entry(name.clone()).or_insert(
            {
                let ns = self.factory.create_scene(name.as_str());
                Rc::new(RefCell::new(ns))
            })
    }

    pub fn get_or_load_scene(&mut self, name : &str) -> &mut Rc<RefCell<scene::Scene>>
    {
        self.scenes.entry(String::from(name)).or_insert(
            {
                let mut ns = scene::Scene::new_from_file(name);

                if let None = ns.camera {
                    let mut cam = self.factory.create_camera();
                    cam.pan(&vec::Vec3::new(-100f64,20f64,100f64));
                    cam.lookat(vec::Vec3::new(0f64,5f64,0f64));
                    ns.camera = Some(Rc::new(RefCell::new(cam)));
                }

                Rc::new(RefCell::new(ns))
            })
    }

    pub fn get_or_load_any_scene(&mut self) -> &mut Rc<RefCell<scene::Scene>>  {
        if self.scenes.is_empty() {
            let files = util::get_files_in_dir("scene");
            if files.is_empty() {
                let s = create_scene_name(String::from("scene/new.scene"));
                self.add_empty_scene(s)
            }
            else {
                self.get_or_load_scene(files[0].to_str().unwrap())
            }
        }
        else {
            let first_key = self.scenes.keys().nth(0).unwrap().clone();
            self.get_or_load_scene(first_key.as_str())
        }
    }
}

pub trait ToId<I : Clone> {
    fn to_id(&self) -> I;
}

pub trait ToId2 {
    type Id;
    fn to_id(&self) -> Self::Id;
}

pub trait GetComponent
{
    fn get_comp<C:Clone+'static>(&self, data : &GetDataT) -> Option<C>;
}

use std::any::Any;
impl GetComponent for Arc<RwLock<object::Object>>
{
    fn get_comp<C:Clone+'static>(&self, data : &GetDataT) -> Option<C>
    {
        let o = self.read().unwrap();
        if let Some(ref mr) = o.mesh_render {
            if let Some(mmr) = (mr as &Any).downcast_ref::<C>() {
                return Some(mmr.clone());
            }
        }

        None
    }
}

pub trait GetDataT{
    fn get_data(&self, id : usize) -> Option<mesh_render::MeshRender>;
}

pub struct NoData;
impl GetDataT for NoData {
    fn get_data(&self, id : usize) -> Option<mesh_render::MeshRender>
    {
        None
    }
}


//TODO remove this
impl GetComponent for usize
{
    fn get_comp<C>(&self, data : &GetDataT) -> Option<C>
    {
        None
    }
}

impl ToId<uuid::Uuid> for Arc<RwLock<object::Object>>
{
    fn to_id(&self) -> uuid::Uuid
    {
        self.read().unwrap().id
    }
}

impl ToId<uuid::Uuid> for Rc<RefCell<scene::Scene>>
{
    fn to_id(&self) -> uuid::Uuid
    {
        self.borrow().id
    }
}

