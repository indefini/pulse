use std::sync::{Arc,RwLock};
use std::rc::{Rc};
use std::cell::{RefCell};
use std::any::Any;

use dormin::{vec, transform,object};
use dormin::property::PropertyGet;
use context;
use operation;
//TODO remove
use ui;

use uuid;

pub struct State
{
    pub context : Box<context::ContextOld>,
    pub op_mgr : operation::OperationManager,
    
    pub saved_positions : Vec<vec::Vec3>,
    pub saved_scales : Vec<vec::Vec3>,
    pub saved_oris : Vec<transform::Orientation>
}

impl State {
    pub fn new() -> State
    {
        State {
            context : box context::Context::new(),
            op_mgr : operation::OperationManager::new(),

            saved_positions : Vec::new(),
            saved_scales : Vec::new(),
            saved_oris : Vec::new()
        }
    }

    pub fn save_positions(&mut self)
    {
        self.saved_positions = self.context.selected.iter().map(|o| o.read().unwrap().position).collect();
    }

    pub fn save_scales(&mut self)
    {
        self.saved_scales = self.context.selected.iter().map(|o| o.read().unwrap().scale).collect();
    }

    pub fn save_oris(&mut self)
    {
        self.saved_oris = self.context.selected.iter().map(|o| o.read().unwrap().orientation).collect();
    }

    pub fn request_operation(
        &mut self,
        name : Vec<String>,
        op_data : operation::OperationData
        ) -> operation::Change
    {
        let op = operation::Operation::new(
            self.context.selected.to_vec(),
            name.clone(),
            op_data
            );

        let change = self.op_mgr.add_with_trait(box op);
        change
    }

    pub fn undo(&mut self) -> operation::Change
    {
        self.op_mgr.undo()
    }

    pub fn redo(&mut self) -> operation::Change
    {
        self.op_mgr.redo()
    }

    pub fn remove_selected_objects(&mut self) -> operation::Change
    {
        println!("state remove sel");

        let s = match self.context.scene {
            Some(ref s) => s.clone(),
            None => return operation::Change::None
        };

        let list = self.context.selected.to_vec();
        let mut vec = Vec::new();
        let mut parent = Vec::new();
        for o in &list {
            vec.push(o.clone());
            let parent_id = if let Some(ref p) = o.read().unwrap().parent {
                p.read().unwrap().id
            }
            else {
                uuid::Uuid::nil()
            };
            parent.push(parent_id);
        }

        let vs = Vec::new();
        return self.request_operation(
            vs,
            operation::OperationData::SceneRemoveObjects(s.clone(), parent, vec)
            );

        //return operation::Change::SceneRemove(s.read().unwrap().id, vec);
    }

    pub fn get_selected_object(&self) -> Option<Arc<RwLock<object::Object>>>
    {
        match self.context.selected.get(0) {
            Some(o) => return Some(o.clone()),
            None => {
                println!("view get selected objects, no objects selected");
                return None;
            }
        };
    }

    pub fn request_operation_vec_del(
        &mut self,
        node : Rc<RefCell<ui::PropertyNode>>
        )
        -> operation::Change
    {
        let node = node.borrow();
        let path = &node.get_path();
        let v: Vec<&str> = path.split('/').collect();

        let mut vs = Vec::new();
        for i in &v
        {
            vs.push(i.to_string());
        }

        let  prop = if let Some(o) = self.get_selected_object(){
            let p : Option<Box<Any>> = o.read().unwrap().get_property_hier(path);
            match p {
                Some(pp) => pp,
                None => return operation::Change::None
            }
        }
        else {
            return operation::Change::None;
        };

        match v[v.len()-1].parse::<usize>() {
            Ok(index) => {
                vs.pop();

                self.request_operation(
                vs,
                operation::OperationData::VecDel(index, prop)
                )
            },
                _ => operation::Change::None
        }
    }

    pub fn request_translation(
        &mut self,
        translation : vec::Vec3) -> operation::Change
    {
        let sp = self.saved_positions.clone();

        for (i,o) in self.context.selected.iter().enumerate() {
            o.write().unwrap().position = sp[i] + translation;
        }

        return operation::Change::DirectChange("position".to_owned());
    }

    pub fn request_scale(
        &mut self,
        scale : vec::Vec3) -> operation::Change
    {
        let sp = self.saved_scales.clone();

        for (i,o) in self.context.selected.iter().enumerate() {
            o.write().unwrap().scale = sp[i] * scale;
        }

        return operation::Change::DirectChange("scale".to_owned());
    }

    pub fn request_rotation(
        &mut self,
        rotation : vec::Quat) -> operation::Change
    {
        let so = self.saved_oris.clone();

        for (i,o) in self.context.selected.iter().enumerate() {
            o.write().unwrap().orientation = so[i] * transform::Orientation::new_with_quat(&rotation);
        }

        operation::Change::DirectChange("orientation/*".to_owned())
    }

    pub fn request_operation_property_old_new_dontcheckequal(
        &mut self,
        property : ui::RefMut<ui::PropertyUser>,
        name : &str,
        old : Box<Any>,
        new : Box<Any>) -> operation::Change
    {
        let op = operation::OldNew::new(
            property,
            String::from(name),
            old,
            new
            );

        let change = self.op_mgr.add_with_trait(box op);
        change
    }

    pub fn request_operation_property_old_new<T : Any+PartialEq>(
        &mut self,
        property : ui::RefMut<ui::PropertyUser>,
        name : &str,
        old : Box<T>,
        new : Box<T>) -> operation::Change
    {
        if *old == *new {
            return operation::Change::None;
        }

        match (&*old as &Any).downcast_ref::<f64>() {
            Some(v) => println!("****************     {}",*v),
            None => {println!("cannot downcast");}
        }

        match (&*new as &Any).downcast_ref::<f64>() {
            Some(v) => println!("****************  nnnnnew    {}",*v),
            None => {println!("cannot downcast");}
        }

        let op = operation::OldNew::new(
            property,
            String::from(name),
            old,
            new
            );

        let change = self.op_mgr.add_with_trait(box op);
        change
    }

    pub fn request_direct_change_property(
        &mut self,
        property : &mut ui::PropertyUser,
        name : &str,
        new : &Any) -> operation::Change
    {
        println!("call from here 00 : {}", name);
        property.test_set_property_hier(name, new);
        operation::Change::DirectChange(String::from(name))
    }

    /*
    pub fn request_operation_option_to_none(
        &mut self,
        property : ui::RefMut<ui::PropertyUser>,
        path : &str,
        old : Box<Any>,
        )
        -> operation::Change
    {
        let op = operation::ToNone::new(
            property,
            String::from(path),
            old);

        let change = self.op_mgr.add_with_trait(box op);
        change
    }

    pub fn request_operation_option_to_some(
        &mut self,
        property : ui::RefMut<ui::PropertyUser>,
        name : &str) -> operation::Change
    {
        let op = operation::ToSome::new(
            property,
            String::from(name));

        let change = self.op_mgr.add_with_trait(box op);
        change
    }
    */

    pub fn request_operation_vec_add(
        &mut self,
        node : Rc<RefCell<ui::PropertyNode>>)
        -> operation::Change
    {
        let nodeb = node.borrow();
        let path = &nodeb.get_path();
        println!("$$$$$$$$$$$$$$$$request operation add vec : {}", path);
        let v: Vec<&str> = path.split('/').collect();

        let mut vs = Vec::new();
        for i in &v
        {
            vs.push(i.to_string());
        }

        let index = match v[v.len()-1].parse::<usize>() {
            Ok(index) => {
                vs.pop();
                index
            },
            _ => 0
        };

            println!("AFTER counts : {}, {}", Rc::strong_count(&node), Rc::weak_count(&node));

        self.request_operation(
            vs,
            operation::OperationData::VecAdd(index)
            )

    }


}

