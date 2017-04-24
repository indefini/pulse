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

impl ToId<uuid::Uuid> for Rc<RefCell<scene::Scene>>
{
    fn to_id(&self) -> uuid::Uuid
    {
        self.borrow().id
    }
}

pub struct Context<S, O, I>
{
    pub selected : Vec<O>,
    pub scene : Option<S>,
    id : PhantomData<I>
}


impl<S : Clone, O, I> Context<S,O,I>
{
    pub fn new() -> Context<S, O, I>
    {
        Context {
            selected: Vec::new(),
            scene : None,
            id : PhantomData
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

    pub fn remove_objects_by_id(&mut self, ids : &[I])
    {
        let mut new_list = Vec::new();
        for o in &self.selected {
            let mut not_found = true;
            for id in ids {
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

impl Context<Rc<RefCell<scene::Scene>>, Arc<RwLock<object::Object>>, uuid::Uuid>
{

    pub fn select_by_id(&mut self, ids : &mut Vec<uuid::Uuid>)
    {
        //TODO same as the code at the end of mouse_up, so factorize
        println!("TODO check: is this find by id ok? : control will try to find object by id, .................select is called ");

        //c.selected.clear();

        let scene = match self.scene {
            Some(ref s) => s.clone(),
            None => return
        };

        let mut obs = scene.borrow().find_objects_by_id(ids);
        self.selected.append(&mut obs);

        //for id in ids.iter() {
            //match scene.read().unwrap().find_object_by_id(id) {
                //Some(o) =>
                    //c.selected.push_back(o.clone()),
                //None => {}
            //};
        //}

    }
}

