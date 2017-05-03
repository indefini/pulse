use std::rc::{Rc};
use std::cell::{RefCell};
use std::collections::HashMap;
use std::fs;

use uuid;

use dormin;
use dormin::{vec, resource, scene, factory};
use dormin::{world};
use context;
use util;
use dormin::input;

static SCENE_SUFFIX: &str = ".scene";
//static WORLD_SUFFIX: &str = ".world";

pub type DataOld = Data<Rc<RefCell<scene::Scene>>>;

pub struct Data<S:SceneT>
{
    pub factory : factory::Factory,
    pub resource : Rc<resource::ResourceGroup>,
    pub scenes : HashMap<String, S>,

    pub worlds : HashMap<String, Box<dormin::world::World>>,
}

pub trait SceneT : ToId<<Self as SceneT>::Id> {
    type Id : Default + Eq;
    fn update(&mut self, dt : f64, input : &input::Input, &resource::ResourceGroup);
}

impl SceneT for Rc<RefCell<scene::Scene>> {
    type Id = uuid::Uuid;
    fn update(&mut self, dt : f64, input : &input::Input, res :&resource::ResourceGroup)
    {
        self.borrow_mut().update(dt, input, res);
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


impl SceneT for world::World {
    type Id = usize;
    fn update(&mut self, dt : f64, input : &input::Input, res :&resource::ResourceGroup)
    {
        println!("TODO !!!!!!!!!!!!!!!!!!!!!!");
    }
}

pub trait DataT<S : SceneT> {
    fn get_scene(&self, id : S::Id) -> Option<&S>;
    fn get_scene_mut(&mut self, id : S::Id) -> Option<&mut S>;
    fn update_scene(&mut self, id : S::Id, input : &input::Input);
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

    fn update_scene(&mut self, id : S::Id, input : &input::Input)
    {
        for s in self.scenes.values_mut() {
            if s.to_id() == id {
                s.update(0.01f64, input, &self.resource);
                return;
            }
        }
    }
}

impl<S:SceneT> Data<S> {

    pub fn new() -> Data<S> {
        Data {
            factory : factory::Factory::new(),
            resource : Rc::new(resource::ResourceGroup::new()),
            scenes : HashMap::new(),

            worlds : HashMap::new(),
        }
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
                let mut ns = scene::Scene::new_from_file(name, &*self.resource);

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

pub fn create_scene_name_with_context(context : &context::ContextOld)
    -> String
{
    let newname = match context.scene {
        Some(ref sc) => {
            let s = sc.borrow();
            let old = if s.name.ends_with(SCENE_SUFFIX) {
                let i = s.name.len() - SCENE_SUFFIX.len();
                let (yep,_) = s.name.split_at(i);
                yep
            }
            else {
                s.name.as_ref()
            };
            String::from(old)
        },
        None => String::from("scene/new.scene")
    };

    create_scene_name(newname)
}

pub fn create_scene_name(name : String) -> String
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

pub trait ToId<I> {
    fn to_id(&self) -> I;
}

pub type ToIdUuid = ToId<uuid::Uuid>;

pub trait ToId2 {
    type Id;
    fn to_id(&self) -> Self::Id;
}


