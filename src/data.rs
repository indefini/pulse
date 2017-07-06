use std::rc::{Rc};
use std::cell::{RefCell};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;

use uuid;

use dormin;
use dormin::{vec, resource, scene, object};
use dormin::mesh_render;
use dormin::render;
use dormin::property::PropertyGet;
use dormin::input;
use dormin::matrix;
use dormin::transform;
use dormin::property::PropertyWrite;
use dormin::resource::ResTT;
use dormin::mesh;

use context;
use util;
use ui::PropertyUser;
use ui::PropertyShow;

use std::path::Path;
use serde_json;
use std::fs::File;
use std::io::{Read,Write};
use std::any::Any;

static SCENE_SUFFIX: &str = ".scene";
//static WORLD_SUFFIX: &str = ".world";


pub struct Data<S:SceneT>
{
    pub scenes : HashMap<String, S>,

    id_count : usize,
}

pub struct MeshTransform
{
    pub mesh : ResTT<mesh::Mesh>,
    pub position : vec::Vec3,
    pub orientation : vec::Quat,
    pub scale : vec::Vec3
}

impl MeshTransform
{
    pub fn with_transform(mesh : ResTT<mesh::Mesh>, transform : &transform::Transform) -> MeshTransform
    {
        MeshTransform {
            mesh : mesh,
            position : transform.position,
            orientation : transform.orientation.as_quat(),
            scale : transform.scale
        }
    }
}

pub trait SceneT : ToId<<Self as SceneT>::Id> + Clone + 'static + PropertyShow {
    type Id : Default + Eq + Clone + Hash + Copy;
    type Object : ToId<Self::Id> + Clone + PropertyGet;
    fn new_empty(name : &str, count : usize) -> Self;
    fn new_from_file(name : &str, count : usize) -> Self;
    fn init_for_play(&mut self, resource : &resource::ResourceGroup);
    fn update(&mut self, dt : f64, input : &input::Input, &resource::ResourceGroup);
    fn get_objects(&self) -> &[Self::Object];
    fn get_objects_vec(&self) -> Vec<Self::Object>
    {
        println!("TODO {}, {}", file!(), line!());
        Vec::new()
    }

    fn get_mmr(&self) -> Vec<render::MatrixMeshRender>
    {
        println!("TODO {}, {}", file!(), line!());
        Vec::new()
    }

    fn get_object_mmr(&self, o : Self::Object) -> Option<render::MatrixMeshRender>
    {
        println!("TODO {}, {}", file!(), line!());
        None
    }

    fn get_object_mt(&self, o : Self::Object) -> Option<MeshTransform>
    {
        println!("TODO {}, {}", file!(), line!());
        None
    }

    fn get_cameras_vec(&self) -> Vec<matrix::Matrix4>
    {
        println!("TODO get cameras_vec {}, {}", file!(), line!());
        Vec::new()
    }

    fn get_camera_obj(&self) -> Option<Self::Object>
    {
        println!("TODO, {}, {}", file!(), line!());
        None
    }

    fn find_objects_with_id(&self, ids : &mut Vec<Self::Id>) -> Vec<Self::Object> {
        println!("TODO {}, {}", file!(), line!());
        Vec::new()
    }

    fn find_object_with_id(&self, id : Self::Id) -> Option<Self::Object> {
        println!("TODO {}, {}", file!(), line!());
        None
    }

    fn get_name(&self) -> String;
    fn set_name(&mut self, s : String);

    fn save(&self);

    fn add_objects(&mut self, parents : &[Option<Self::Id>], obs : &[Self::Object])
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

    fn set_position(&mut self, o : Self::Object, v : vec::Vec3)
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

    fn get_transform(&self, o: Self::Object) -> transform::Transform;
    fn get_world_transform(&self, o: Self::Object) -> transform::Transform;

    //TODO use &str instead of string?
    fn get_object_name(&self, o : Self::Object) -> String
    {
        println!("TODO {}, {}", file!(), line!());
        //"TODO name, yo".to_owned()
        format!("TODO {}, {}", file!(), line!())
    }

    fn get_comp_data_value<T:Any+Clone>(&self, o : Self::Object) -> Option<T>
    {
        println!("TODO, {}, {}", file!(), line!());
        None
    }

    fn create_empty_object(&mut self, name : &str) -> Self::Object;
    fn create_empty_object_at_position(&mut self, name : &str, v : vec::Vec3) -> Self::Object;
    fn create_empty_object_string(&mut self, name : &str) -> String
    {
        println!("TODO {}, {}", file!(), line!());
        String::new()
    }

    fn get_property_write_from_object(&mut self, o : Self::Object, name :&str) 
        -> Option<(&mut PropertyWrite, String)>;

}

impl SceneT for Rc<RefCell<scene::Scene>> {
    type Id = uuid::Uuid;
    type Object = Arc<RwLock<object::Object>>;

    fn new_empty(name : &str, count : usize) -> Self
    {
        let s = scene::Scene::new(name, uuid::Uuid::new_v4(), dormin::camera::Camera::new());
        Rc::new(RefCell::new(s))
    }
    
    fn new_from_file(name : &str, count : usize) -> Self
    {
        let s = scene::Scene::new_from_file(name);
        /*
                if let None = s.camera {
                    let mut cam = self.factory.create_camera();
                    cam.pan(&vec::Vec3::new(-100f64,20f64,100f64));
                    cam.lookat(vec::Vec3::new(0f64,5f64,0f64));
                    ns.camera = Some(Rc::new(RefCell::new(cam)));
                }
                */

        Rc::new(RefCell::new(s))
    }

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

    fn find_objects_with_id(&self, ids : &mut Vec<Self::Id>) -> Vec<Self::Object> {
        self.borrow().find_objects_by_id(ids)
    }

    fn find_object_with_id(&self, id : Self::Id) -> Option<Self::Object>
    {
        self.borrow().find_object_with_id(&id)
    }

    fn get_name(&self) -> String
    {
        self.borrow().name.clone()
    }

    fn set_name(&mut self, s : String)
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

    fn add_objects(&mut self, parents : &[Option<Self::Id>], obs : &[Self::Object])
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

    fn set_position(&mut self, o : Self::Object, v : vec::Vec3)
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

    fn get_transform(&self, o: Self::Object) -> transform::Transform
    {
        o.read().unwrap().make_transform()
    }
    fn get_world_transform(&self, o: Self::Object) -> transform::Transform
    {
        o.read().unwrap().make_world_transform()
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

    fn create_empty_object_at_position(&mut self, name : &str, v : vec::Vec3) -> Self::Object
    {
       let mut o = object::Object::new(name);
       o.position = v;
       Arc::new(RwLock::new(o))
    }

    fn get_property_write_from_object(&mut self, o : Self::Object, name :&str) 
        -> Option<(&mut PropertyWrite, String)>
    {
        println!("TODO {}, {}", file!(), line!());
        //Some((&mut o.clone(), name.to_owned()))
        None
    }

    fn get_object_mt(&self, o : Self::Object) -> Option<MeshTransform>
    {
        let ob = o.read().unwrap();
        ob.mesh_render.as_ref().map(
            |x| MeshTransform::with_transform(x.mesh.clone(), &ob.make_world_transform()))
    }

    fn get_object_mmr(&self, o : Self::Object) -> Option<render::MatrixMeshRender>
    {
        let ob = o.read().unwrap();
        ob.mesh_render.as_ref().map(
            |x| render::MatrixMeshRender::new(ob.get_world_matrix(),x.clone()))
    }


}

impl<S:SceneT> Data<S> {

    pub fn new() -> Data<S> {
        Data {
            scenes : HashMap::new(),

            id_count : 0usize
        }
    }

    pub fn get_scene(&self, id : S::Id) -> Option<&S>
    {
        for v in self.scenes.values() {
            if v.to_id() == id {
                return Some(v)
            }
        }

        None
    }

    pub fn get_scene_mut(&mut self, id : S::Id) -> Option<&mut S>
    {
        for v in self.scenes.values_mut() {
            if v.to_id() == id {
                return Some(v)
            }
        }

        None
    }

    pub fn add_empty_scene(&mut self, name : String) -> &mut S
    {
        self.id_count +=1;
        self.scenes.entry(name.clone()).or_insert(
                S::new_empty(&name, self.id_count)
            )
    }

    pub fn get_or_load_scene(&mut self, name : &str) -> &mut S
    {
        //todo
        self.id_count +=1;
        self.scenes.entry(String::from(name)).or_insert(
            S::new_from_file(name, self.id_count))
    }

    pub fn get_or_load_any_scene(&mut self) -> &mut S
    {
        //TODO
        self.id_count +=1;

        if self.scenes.is_empty() {
            let files = util::get_files_in_dir("scene");
            if files.is_empty() {
                self.add_empty_scene(String::from("scene/new.scene"))
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

    pub fn create_scene_name_with_context(&self, context : &context::Context<S>)
    -> String
    {
        let newname = match context.scene {
            Some(ref sc) => {
                let scene = self.get_scene(sc.clone()).unwrap();
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

    pub fn getP_copy(&mut self, id : S::Id) -> Option<Box<PropertyWrite>>
    {
        println!("TODO or erase {}, {}", file!(), line!());
        None
    }

    pub fn get_property_write(
        &mut self, 
        scene_id : S::Id,
        object_id : S::Id,
        property : &str)
        -> Option<(&mut PropertyWrite, String)>
    {
        for s in self.scenes.values_mut() {
            if s.to_id() != scene_id {
                //continue;
            }

            if property == "position" {
                if let Some(ref o) = s.find_object_with_id(object_id) {
            println!("TODO or erase {}, {}, {}", property, file!(), line!());
                   //return s.get_comp_mut::<transform::Transform>(&o.to_mut()).map(|x| (x as &mut PropertyWrite,property.to_owned()));
                   return s.get_property_write_from_object(o.clone(), property);
                }
            }

        }

        None
    }

    pub fn add_objects(
        &mut self,
        scene_id : S::Id,
        parents : &[Option<S::Id>],
        objects : &[S::Object])
    {
        if let Some(s) = self.get_scene_mut(scene_id) {
            s.add_objects(parents, objects);
        }
    }

    pub fn remove_objects(
        &mut self,
        scene_id : S::Id,
        parents : &[Option<S::Id>],
        objects : &[S::Object])
    {
        if let Some(s) = self.get_scene(scene_id) {
            s.remove_objects(parents, objects);
        }
    }

    pub fn set_camera(
        &mut self, 
        scene_id : S::Id,
        camera : Option<S::Object>)
    {
        if let Some(s) = self.get_scene(scene_id) {
            s.set_camera(camera);
        }
    }

    pub fn get_property_user_copy(&self, id : S::Id) -> Option<Box<PropertyUser<S>>>
    {
        println!("TODO, change this function to include scene id too {}, {}", file!(), line!());
        for s in self.scenes.values() {
            if s.to_id() == id {
                println!("TODO {}, {}", file!(), line!());
                //return Some(box s.clone());
            }

            for o in s.get_objects() {
                if o.to_id() == id {
                println!("TODO {}, {}", file!(), line!());
                    //return Some(box o.clone());
                }
            }

        }

        None
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

pub trait ToId<I : Clone> {
    fn to_id(&self) -> I;
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

