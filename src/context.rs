use dormin::object;
use dormin::scene;
use dormin::vec;
use uuid;
use dormin::transform;

use std::collections::{LinkedList};
use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::cell::{RefCell, BorrowState};
use std::marker::PhantomData;

pub type ContextOld = Context<Rc<RefCell<scene::Scene>>, Arc<RwLock<object::Object>>, uuid::Uuid>;

trait ToId<I> {
    fn to_id(&self) -> I;
}

impl ToId<uuid::Uuid> for Arc<RwLock<object::Object>>
{
    fn to_id(&self) -> uuid::Uuid 
    {
        self.read().unwrap().id
    }
}

pub struct Context<S, O, I>
{
    pub selected : Vec<O>,
    pub scene : Option<S>,
    phantom : PhantomData<I>
}


impl<S : Clone, O, I> Context<S,O,I>
{
    pub fn new() -> Context<S, O, I>
    {
        Context {
            selected: Vec::new(),
            scene : None,
            phantom : PhantomData
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

impl<S, O : ToId<I> + Clone, I : Eq> Context<S, O, I>
{
    pub fn get_vec_selected_ids(&self) -> Vec<I>
    {
        let mut v = Vec::with_capacity(self.selected.len());
        for o in &self.selected {
            v.push(o.to_id());
        }

        v
    }

    pub fn remove_objects_by_id(&mut self, ids : Vec<I>)
    {
        let mut new_list = Vec::new();
        for o in &self.selected {
            let mut not_found = true;
            for id in &ids {
                if *id == o.to_id() {
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

    pub fn has_object_with_id(&self, id : &I) -> bool
    {
        for o in &self.selected {
            if *id == o.to_id() {
               return true;
            }
        }

        false
    }

    pub fn has_object(&self, ob : O) -> bool
    {
        for o in &self.selected {
            if ob.to_id() == o.to_id() {
               return true;
            }
        }

        false
    }

}

