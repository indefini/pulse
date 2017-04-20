use dormin::object;
use dormin::scene;
use dormin::vec;
use uuid;
use dormin::transform;

use std::collections::{LinkedList};
use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::cell::{RefCell, BorrowState};

pub type ContextOld = Context<Rc<RefCell<scene::Scene>>, Arc<RwLock<object::Object>>>;

pub struct Context<S, O>
{
    pub selected : Vec<O>,
    pub scene : Option<S>,
}


impl<S : Clone, O> Context<S,O>
{
    pub fn new() -> Context<S, O>
    {
        Context {
            selected: Vec::new(),
            scene : None,
        }
    }

    pub fn set_scene(&mut self, scene : S)
    {
        self.scene = Some(scene);
        self.selected.clear();
    }

    pub fn get_scene(&self) -> Option<S>
    {
        self.scene.clone()
    }

}


impl Context<Rc<RefCell<scene::Scene>>,Arc<RwLock<object::Object>>>
{
    pub fn get_vec_selected_ids(&self) -> Vec<uuid::Uuid>
    {
        let mut v = Vec::with_capacity(self.selected.len());
        for o in &self.selected {
            v.push(o.read().unwrap().id.clone());
        }

        v
    }

    pub fn remove_objects_by_id(&mut self, ids : Vec<uuid::Uuid>)
    {
        let mut new_list = Vec::new();
        for o in &self.selected {
            let mut not_found = true;
            for id in &ids {
                if *id == o.read().unwrap().id {
                    not_found = false;
                    break;
                }
            }
            if not_found {
                new_list.push(o.clone());
            }
        }

        self.selected = new_list;
    }

    pub fn add_objects_by_id(&mut self, ids : Vec<uuid::Uuid>)
    {
        for id in &ids {
            let mut found = false;
            for o in &self.selected {
                if *id == o.read().unwrap().id {
                    found = true;
                    break;
                }
            }
            if !found {
                if let Some(ref s) = self.scene {
                    for so in &s.borrow().objects {
                        if *id == so.read().unwrap().id {
                            self.selected.push(so.clone());
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn has_object_with_id(&self, id : &uuid::Uuid) -> bool
    {
        for o in &self.selected {
            if *id == o.read().unwrap().id {
               return true;
            }
        }

        false
    }

    pub fn has_object(&self, ob : &object::Object) -> bool
    {
        for o in &self.selected {
            if ob.id == o.read().unwrap().id {
               return true;
            }
        }

        false
    }

}

