use std::rc::{Rc};
use std::cell::{RefCell};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::Path;
use std::fs;
use std::fs::File;
use serde_json;
use std::io::{Read,Write};

use uuid::Uuid;

use dormin;
use dormin::vec;
use dormin::resource;
use dormin::scene;
use dormin::object;
use dormin::factory;
use context;
use uuid;
use util;

static SCENE_SUFFIX: &'static str = ".scene";


pub struct Data<S>
{
    pub factory : factory::Factory,
    pub resource : Rc<resource::ResourceGroup>,
    pub scenes : HashMap<String, S>,

    pub worlds : HashMap<String, Box<dormin::world::World>>,
}

impl Data<Rc<RefCell<scene::Scene>>> {
    pub fn new() -> Data<Rc<RefCell<scene::Scene>>> {
        Data {
            factory : factory::Factory::new(),
            resource : Rc::new(resource::ResourceGroup::new()),
            scenes : HashMap::new(),

            worlds : HashMap::new(),
        }
    }

    pub fn add_empty_scene(&mut self, name : String) -> Rc<RefCell<scene::Scene>>
    {
        self.scenes.entry(name.clone()).or_insert(
            {
                let ns = self.factory.create_scene(name.as_str());
                Rc::new(RefCell::new(ns))
            }).clone()
    }

    pub fn get_or_load_scene(&mut self, name : &str) -> Rc<RefCell<scene::Scene>>
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
            }).clone()
    }

    pub fn get_or_load_any_scene(&mut self) -> Rc<RefCell<scene::Scene>>  {
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

