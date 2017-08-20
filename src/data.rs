use std::rc::{Rc};
use std::cell::{RefCell};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::fmt::Debug;

use dormin::{vec, resource};
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
use ui::PropertyId;

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
    type Id : Default + Eq + Clone + Hash + Copy + Debug;
    type Object : ToId<Self::Id> + Clone + PropertyGet + Debug; //TODO remove PropertyGet
    fn new_empty(name : &str, count : usize) -> Self;
    fn new_from_file(name : &str, count : usize) -> Self;
    fn init_for_play(&mut self, resource : &resource::ResourceGroup);
    fn update(&mut self, dt : f64, input : &input::Input, &resource::ResourceGroup);
    fn get_objects(&self) -> &[Self::Object];
    fn get_objects_vec(&self) -> Vec<Self::Object>
    {
        unimplemented!("TODO {}, {}", file!(), line!());
    }

    fn get_mmr(&self) -> Vec<render::MatrixMeshRender>
    {
        unimplemented!("TODO {}, {}", file!(), line!());
    }

    fn get_object_mmr(&self, o : Self::Object) -> Option<render::MatrixMeshRender>
    {
        unimplemented!("TODO {}, {}", file!(), line!());
    }

    fn get_object_mt(&self, o : Self::Object) -> Option<MeshTransform>
    {
        unimplemented!("TODO {}, {}", file!(), line!());
    }

    fn get_cameras_vec(&self) -> Vec<matrix::Matrix4>
    {
        println!("TODO get cameras_vec {}, {}", file!(), line!());
        Vec::new()
    }

    fn get_camera_obj(&self) -> Option<Self::Object>
    {
        unimplemented!("TODO, {}, {}", file!(), line!());
    }

    fn find_objects_with_id(&self, ids : &mut Vec<Self::Id>) -> Vec<Self::Object> {
        unimplemented!("TODO {}, {}", file!(), line!());
    }

    fn find_object_with_id(&self, id : Self::Id) -> Option<Self::Object> {
        unimplemented!("TODO {}, {}", file!(), line!());
    }

    fn get_name(&self) -> String;
    fn set_name(&mut self, s : String);

    fn save(&mut self);

    fn add_objects(&mut self, parents : &[Option<Self::Id>], obs : &[Self::Object])
    {
        println!("TODO, {}, {}", file!(), line!());
    }

    fn remove_objects(&mut self, parents : &[Option<Self::Id>], obs : &[Self::Object])
    {
        unimplemented!("TODO, {}, {}", file!(), line!());
    }

    fn set_camera(&self, ob : Option<Self::Object>)
    {
        unimplemented!("TODO, {}, {}", file!(), line!());
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
        panic!("TODO, {}, {}", file!(), line!());
    }

    fn set_scale(&mut self, o : Self::Object, v : vec::Vec3)
    {
        panic!("TODO, {}, {}", file!(), line!());
    }

    fn set_orientation(&mut self, o : Self::Object, ori : transform::Orientation)
    {
        panic!("TODO, {}, {}", file!(), line!());
    }

    fn get_position(&self, o : Self::Object) -> vec::Vec3
    {
        panic!("TODO, {}, {}", file!(), line!());
    }

    fn get_scale(&self, o : Self::Object) -> vec::Vec3
    {
        panic!("TODO, {}, {}", file!(), line!());
    }

    fn get_orientation(&self, o : Self::Object) -> transform::Orientation
    {
        panic!("TODO, {}, {}", file!(), line!());
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
    
    fn get_property_write_from_object_copy(&mut self, o : Self::Object, name :&str) 
        -> Option<(Box<PropertyWrite>, String)>
        {
            unimplemented!()
        }

    fn get_property_show_from_object_copy(&self, o : Self::Object, name :&str) 
        -> Option<(Box<PropertyShow>, String)>
        {
            unimplemented!()
        }
    fn get_property_show_from_object_copy_vec(&self, o : Self::Object) 
        -> Vec<Box<PropertyShow>>
        {
            unimplemented!()
        }

    fn get_property_show_from_object_copy_hash(&self, o : Self::Object)
        -> HashMap<String, Box<PropertyShow>>
        {
            unimplemented!()
        }

    fn add_component(&mut self, o : Self::Object, compopent : &str)
    {
       unimplemented!()
    }

    fn remove_component(&mut self, o : Self::Object, compopent : &str)
    {
        unimplemented!()
    }

    fn get_existing_components() -> Vec<&'static str>
    {
        unimplemented!()
    }

    fn get_full_object(&self, o : Self::Object) -> Self::Object
    {
        unimplemented!()
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

    pub fn get_property_write_copy(&mut self, object_id : S::Id, property : &str) 
        -> Option<(Box<PropertyWrite>, String)>
    {
        println!("TODO or erase {}, {}", file!(), line!());
        //use backtrace::Backtrace;
        //let bt = Backtrace::new();
        //println!("bbbbbb : {:?}", bt);
        for s in self.scenes.values_mut() {
            /*
            if s.to_id() != scene_id {
                //continue;
            }
            */

            if let Some(ref o) = s.find_object_with_id(object_id) {
                return s.get_property_write_from_object_copy(o.clone(), property);
            }

        }
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

            if let Some(ref o) = s.find_object_with_id(object_id) {
                return s.get_property_write_from_object(o.clone(), property);
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
        if let Some(s) = self.get_scene_mut(scene_id) {
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

    /*
    pub fn get_property_user_copy(&self, id : S::Id) -> Option<Box<PropertyUser<S>>>
    {
        println!("TODO, change this function to include scene id too, scenes : {}, {}, {}", self.scenes.len(), file!(), line!());
        for s in self.scenes.values() {
            if s.to_id() == id {
                println!("TODO {}, {}", file!(), line!());
                //return Some(box s.clone());
            }

            println!("object len {}", s.get_objects().len());

            for o in s.get_objects() {
                println!("Testing object {:?}, {:?}", id, o.to_id());
                if o.to_id() == id {
                println!("TODO {}, {}", file!(), line!());
                    //return Some(box o.clone());
                }
            }

        }

        None
    }
    */

    /*
    pub fn get_property_show_copy(&self, id : S::Id) -> Option<Box<PropertyShow>>
        //where S::Object : PropertyId<S::Id>
    {
        println!("TODO, change this function to include scene id too, scenes : {}, {}, {}", self.scenes.len(), file!(), line!());
        for s in self.scenes.values() {
            if s.to_id() == id {
                println!("TODO {}, {}", file!(), line!());
                //return Some(box s.clone());
            }

            println!("object len {}", s.get_objects().len());

            for o in s.get_objects() {
                println!("Testing object {:?}, {:?}", id, o.to_id());
                if o.to_id() == id {
                println!("TODO {}, {}", file!(), line!());
                    return Some(box o.clone());
                }
            }

        }

        None
    }
    */

    //pub fn get_property_show_copy(&mut self, object_id : S::Id, property : &str) 
        //-> Option<(Box<PropertyShow>, String)>
    pub fn get_property_show_copy(&self, object_id : S::Id) 
        -> Option<Box<PropertyShow>>
    {
        println!("TODO or erase {}, {}", file!(), line!());
        for s in self.scenes.values() {
            /*
            if s.to_id() != scene_id {
                //continue;
            }
            */

            if let Some(ref o) = s.find_object_with_id(object_id) {
                return s.get_property_show_from_object_copy(o.clone(), "").map(|x| x.0);
            }

        }
        None
    }

    pub fn get_property_show_copy_vec(&self, object_id : S::Id)
        -> Vec<Box<PropertyShow>>
    {
        println!("TODO or erase {}, {}", file!(), line!());
        for s in self.scenes.values() {
            /*
            if s.to_id() != scene_id {
                //continue;
            }
            */

            if let Some(ref o) = s.find_object_with_id(object_id) {
                return s.get_property_show_from_object_copy_vec(o.clone())
            }

        }

        vec![]
    }

    pub fn get_property_show_copy_hash(&self, object_id : S::Id)
        -> HashMap<String, Box<PropertyShow>>
    {
        println!("TODO or erase {}, {}", file!(), line!());
        for s in self.scenes.values() {
            /*
            if s.to_id() != scene_id {
                //continue;
            }
            */

            if let Some(ref o) = s.find_object_with_id(object_id) {
                return s.get_property_show_from_object_copy_hash(o.clone())
            }

        }

        HashMap::new()
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

impl<I:Clone, T: ToId<I>> ToId<I> for Rc<RefCell<T>>
{
    fn to_id(&self) -> I
    {
        self.borrow().to_id()
    }

}

impl<I:Clone, T: ToId<I>> ToId<I> for Arc<RwLock<T>>
{
    fn to_id(&self) -> I
    {
        self.read().unwrap().to_id()
    }
}


//impl ToId<uuid::Uuid> for Arc<RwLock<Object>>

impl<S:SceneT> SceneT for Rc<RefCell<S>> {
    type Id = S::Id;
    type Object = S::Object;

    fn new_empty(name : &str, count : usize) -> Self
    {
        let s = S::new_empty(name, count);
        Rc::new(RefCell::new(s))
    }
    
    fn new_from_file(name : &str, count : usize) -> Self
    {
        let s = S::new_from_file(name, count);
        Rc::new(RefCell::new(s))
    }

    fn update(&mut self, dt : f64, input : &input::Input, res :&resource::ResourceGroup)
    {
        self.borrow_mut().update(dt, input, res);
    }

    fn init_for_play(&mut self, resource : &resource::ResourceGroup)
    {
        self.borrow_mut().init_for_play(resource);
    }

    fn get_objects(&self) -> &[Self::Object]
    {
        unimplemented!();
        &[]//&self.borrow().objects
    }

    fn get_objects_vec(&self) -> Vec<Self::Object>
    {
        self.borrow().get_objects_vec()
    }

    fn find_objects_with_id(&self, ids : &mut Vec<Self::Id>) -> Vec<Self::Object> {
        self.borrow().find_objects_with_id(ids)
    }

    fn find_object_with_id(&self, id : Self::Id) -> Option<Self::Object>
    {
        self.borrow().find_object_with_id(id)
    }

    fn get_name(&self) -> String
    {
        self.borrow().get_name()
    }

    fn set_name(&mut self, s : String)
    {
        self.borrow_mut().set_name(s);
    }

    fn save(&mut self)
    {
        self.borrow_mut().save();
    }

    fn get_cameras_vec(&self) -> Vec<matrix::Matrix4>
    {
        self.borrow().get_cameras_vec()
    }

    fn get_camera_obj(&self) -> Option<Self::Object>
    {
        self.borrow().get_camera_obj()
    }

    fn get_mmr(&self) -> Vec<render::MatrixMeshRender>
    {
        self.borrow().get_mmr()
    }

    fn add_objects(&mut self, parents : &[Option<Self::Id>], obs : &[Self::Object])
    {
        self.borrow_mut().add_objects(parents, obs);
    }

    fn remove_objects(&mut self, parents : &[Option<Self::Id>], obs : &[Self::Object])
    {
        self.borrow_mut().remove_objects(parents, obs);
    }

    fn set_camera(&self, ob : Option<Self::Object>)
    {
        self.borrow().set_camera(ob)
    }

    fn get_parent(&self, o : Self::Object) -> Option<Self::Object>
    {
        self.borrow().get_parent(o)
    }

    fn get_children(&self, o : Self::Object) -> Vec<Self::Object>
    {
        self.borrow().get_children(o)
    }

    fn set_position(&mut self, o : Self::Object, v : vec::Vec3)
    {
        self.borrow_mut().set_position(o, v);
    }

    fn set_scale(&mut self, o : Self::Object, v : vec::Vec3)
    {
        self.borrow_mut().set_scale(o, v);
    }

    fn set_orientation(&mut self, o : Self::Object, ori : transform::Orientation)
    {
        self.borrow_mut().set_orientation(o, ori);
    }

    fn get_position(&self, o : Self::Object) -> vec::Vec3
    {
        self.borrow().get_position(o)
    }

    fn get_scale(&self, o : Self::Object) -> vec::Vec3
    {
        self.borrow().get_scale(o)
    }

    fn get_orientation(&self, o : Self::Object) -> transform::Orientation
    {
        self.borrow().get_orientation(o)
    }

    fn get_transform(&self, o: Self::Object) -> transform::Transform
    {
        self.borrow().get_transform(o)
    }
    fn get_world_transform(&self, o: Self::Object) -> transform::Transform
    {
        self.borrow().get_world_transform(o)
    }

    fn get_object_name(&self, o : Self::Object) -> String
    {
        self.borrow().get_object_name(o)
    }

    fn get_comp_data_value<T:Any+Clone>(&self, o : Self::Object) -> Option<T>
    {
        self.borrow().get_comp_data_value(o)
    }

    fn create_empty_object(&mut self, name : &str) -> Self::Object
    {
        self.borrow_mut().create_empty_object(name)
    }

    fn create_empty_object_at_position(&mut self, name : &str, v : vec::Vec3) -> Self::Object
    {
        self.borrow_mut().create_empty_object_at_position(name, v)
    }

    fn get_property_write_from_object(&mut self, o : Self::Object, name :&str) 
        -> Option<(&mut PropertyWrite, String)>
    {
        //self.borrow_mut().get_property_write_from_object(o, name)
        unimplemented!()
    }


    fn get_object_mt(&self, o : Self::Object) -> Option<MeshTransform>
    {
        self.borrow().get_object_mt(o)
    }

    fn get_object_mmr(&self, o : Self::Object) -> Option<render::MatrixMeshRender>
    {
        self.borrow().get_object_mmr(o)
    }


}

