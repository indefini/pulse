use std::rc::{Rc};
use std::cell::{RefCell};
use std::any::Any;

use dormin::{vec, transform};
use dormin::property::{PropertyGet,PropertyWrite};

use context;
use operation;
use data::{Data, SceneT,ToId};

use ui;

/*
struct WorldChange
{
    ob : dormin::world::EntityRef,
    what : String,
    value : vec::Vec3
}
*/


pub struct State<S:SceneT>
{
    pub context : Box<context::Context<S>>,
    pub op_mgr : operation::OperationManager<S>,
    
    pub saved_positions : Vec<vec::Vec3>,
    pub saved_scales : Vec<vec::Vec3>,
    pub saved_oris : Vec<transform::Orientation>
}

impl<S:SceneT+'static> State<S> {
    pub fn new() -> State<S>
    {
        State {
            context : box context::Context::new(),
            op_mgr : operation::OperationManager::new(),

            saved_positions : Vec::new(),
            saved_scales : Vec::new(),
            saved_oris : Vec::new()
        }
    }

    pub fn save_transforms(&mut self, data : &Data<S>)
    {
        let sid = self.context.scene.as_ref().unwrap();

        let scene = data.get_scene(sid.clone()).unwrap();
        self.saved_positions =
            self.context.selected.iter().map(
                |o| scene.get_position(o.clone())
                ).collect();

        self.saved_scales = self.context.selected.iter().map(|o| scene.get_scale(o.clone())).collect();
        self.saved_oris = self.context.selected.iter().map(|o| scene.get_orientation(o.clone())).collect();
        
    }

    pub fn make_operation(
        &mut self,
        name : Vec<String>,
        op_data : operation::OperationData<S>
        ) -> operation::Operation<S>
    {
        let obs : Vec<Box<S::Object>> =
            self.context.selected.iter().map(|x| (box x.clone())).collect();

        operation::Operation::new(
            obs,
            name.clone(),
            op_data
            )
    }

    pub fn request_operation(
        &mut self,
        name : Vec<String>,
        op_data : operation::OperationData<S>,
        rec : &mut Data<S>
        ) -> operation::Change<S::Id>
    {
        let op = self.make_operation(name, op_data);
        let change = self.op_mgr.add_with_trait(box op, rec);
        change
    }

    /*
    pub fn request_operation(
        &mut self,
        name : Vec<String>,
        op_data : operation::OperationDataOld
        ) -> operation::Change
    {
        let obs : Vec<Box<ToIdUuid>> =
            self.context.selected.iter().map(|x| (box x.clone()) as Box<ToIdUuid>).collect();

        let op = operation::Operation::new(
            //self.context.selected.iter().map(|x| box x.clone()).collect(),
            obs,
            name.clone(),
            op_data
            );

        let change = self.op_mgr.add_with_trait(box op);
        change
    }
    */

    pub fn undo(&mut self, rec : &mut Data<S>) -> operation::Change<S::Id>
    {
        self.op_mgr.undo(rec)
    }

    pub fn redo(&mut self, rec : &mut Data<S>) -> operation::Change<S::Id>
    {
        self.op_mgr.redo(rec)
    }

    pub fn remove_selected_objects(
        &mut self,
        data : &mut Data<S>,
        ) -> operation::Change<S::Id>
    {
        println!("state remove sel");


        let sid = if let Some(ref s) = self.context.scene {
            s.clone()
        }
        else {
            return operation::Change::None;
        };

        let vec : Vec<S::Object> = {
            let s = data.get_scene(sid.clone()).unwrap();
            //self.context.selected.to_vec();
            self.context.selected.iter().map(|x| s.get_full_object(x.clone())).collect()
        };

        let parent =
        {
            let s = data.get_scene(sid.clone()).unwrap();
            vec.iter().map(|o| s.get_parent(o.clone()).map(|x| x.to_id())).collect()
        };

        println!("vec : {:?}", vec);

        return self.request_operation(
            vec![],
            operation::OperationData::SceneRemoveObjects(sid, parent, vec),
            data
            );
    }

    pub fn get_selected_object(&self) -> Option<S::Object>
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
        node : Rc<RefCell<ui::PropertyNode>>,
        rec : &mut Data<S>
        )
        -> operation::Change<S::Id>
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
            let p : Option<Box<Any>> = o.get_property_hier(path);
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
                operation::OperationData::VecDel(index, prop),
                rec
                )
            },
                _ => operation::Change::None
        }
    }

    pub fn request_translation(
        &mut self,
        data : &mut Data<S>,
        translation : vec::Vec3) -> operation::Change<S::Id>
    {
        let s = data.get_scene_mut(self.context.scene.as_ref().unwrap().clone()).unwrap();

        let sp = self.saved_positions.clone();

        for (i,o) in self.context.selected.iter().enumerate() {
            s.set_position(o.clone(), sp[i] + translation);
        }

        return operation::Change::DirectChange("transform/position".to_owned());
    }

    pub fn request_scale(
        &mut self,
        data : &mut Data<S>,
        scale : vec::Vec3) -> operation::Change<S::Id>
    {
        let s = data.get_scene_mut(self.context.scene.as_ref().unwrap().clone()).unwrap();
        let sp = self.saved_scales.clone();

        for (i,o) in self.context.selected.iter().enumerate() {
            s.set_scale(o.clone(), sp[i] * scale);
        }

        return operation::Change::DirectChange("transform/scale".to_owned());
    }

    pub fn request_rotation(
        &mut self,
        data : &mut Data<S>,
        rotation : vec::Quat) -> operation::Change<S::Id>
    {
        let s = data.get_scene_mut(self.context.scene.as_ref().unwrap().clone()).unwrap();
        let so = self.saved_oris.clone();

        for (i,o) in self.context.selected.iter().enumerate() {
            s.set_orientation(o.clone(), so[i] * transform::Orientation::new_with_quat(&rotation));
        }

        operation::Change::DirectChange("transform/orientation/*".to_owned())
    }

    pub fn request_operation_property_old_new_dontcheckequal(
        &mut self,
        id : S::Id,
        name : &str,
        old : Box<Any>,
        new : Box<Any>,
        rec : &mut Data<S>
        ) -> operation::Change<S::Id>
    {
        let op : operation::OldNew<S> = operation::OldNew::new(
            id,
            String::from(name),
            old,
            new
            );

        let change = self.op_mgr.add_with_trait(box op, rec);
        change
    }

    pub fn request_operation_property_old_new<T : Any+PartialEq>(
        &mut self,
        id : S::Id,
        name : &str,
        old : Box<T>,
        new : Box<T>,
        rec : &mut Data<S>,
        ) -> operation::Change<S::Id>
    {
        if *old == *new {
            return operation::Change::None;
        }

        {
        // DEBUG, can erase
        match (&*old as &Any).downcast_ref::<f64>() {
            Some(v) => println!("****************     {}",*v),
            None => {println!("cannot downcast");}
        }

        match (&*new as &Any).downcast_ref::<f64>() {
            Some(v) => println!("****************  nnnnnew    {}",*v),
            None => {println!("cannot downcast");}
        }
        }

        let op : operation::OldNew<S> = operation::OldNew::new(
            id,
            String::from(name),
            old,
            new
            );

        let change = self.op_mgr.add_with_trait(box op, rec);
        change
    }

    pub fn request_direct_change_property(
        &mut self,
        //property : &mut ui::PropertyUser<S>,
        property : &mut PropertyWrite,
        name : &str,
        new : &Any) -> operation::Change<S::Id>
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
        node : Rc<RefCell<ui::PropertyNode>>,
        rec : &mut Data<S>
        )
        -> operation::Change<S::Id>
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
            operation::OperationData::VecAdd(index),
            rec
            )

    }

    pub fn copy_selected_objects(
        &mut self,
        //TODO factory : &factory::Factory,
        data : &mut Data<S>,
        ) -> operation::Change<S::Id>
    {
        let sid = if let Some(ref s) = self.context.scene {
            s.clone()
        }
        else {
            return operation::Change::None;
        };

        let mut vec = Vec::new();
        let mut parents = Vec::new();
        {
            let s = data.get_scene(sid.clone()).unwrap();

            for o in &self.context.selected {
                println!("COPY is not working because of this TODO");
                //TODO vec.push(Arc::new(RwLock::new(factory.copy_object(&*ob))));
                let parent_id = s.get_parent(o.clone()).map(|x| x.to_id());

                parents.push(parent_id);
            }
        }

        let vs = Vec::new();
        return self.request_operation(
            vs,
            operation::OperationData::SceneAddObjects(sid, parents, vec),
            data
            );
    }

    pub fn add_component(
        &mut self,
        component_name : &str,
        rec : &mut Data<S>
        ) -> operation::Change<S::Id>
    {
        println!("!!!!!!!!!!!!!!!!!!!TODO add component {} {}", file!(), line!());
        let o = if let Some(o) = self.context.selected.get(0) {
            o.clone()
        }
        else
        {
            println!("TODO display error message, or set this as cannot be executed command if nothing is selected {} {}", file!(), line!());
            return operation::Change::None;
        };

        let s = if let Some(s) = self.context.scene {
            s.clone()
        }
        else
        {
            return operation::Change::None;
        };

        let vs = Vec::new();

        self.request_operation(
            vs,
            operation::OperationData::AddComponent(s.clone(), o.clone(), component_name.to_owned()),
            rec
            )
    }

    pub fn remove_component(
        &mut self,
        component_name : &str,
        rec : &mut Data<S>
        ) -> operation::Change<S::Id>
    {
        println!("!!!!!!!!!!!!!!!!!!!TODO add component {} {}", file!(), line!());
        let o = if let Some(o) = self.context.selected.get(0) {
            o.clone()
        }
        else
        {
            println!("TODO display error message, or set this as cannot be executed command if nothing is selected {} {}", file!(), line!());
            return operation::Change::None;
        };

        let s = if let Some(s) = self.context.scene {
            s.clone()
        }
        else
        {
            return operation::Change::None;
        };

        let vs = Vec::new();

        self.request_operation(
            vs,
            operation::OperationData::RemoveComponent(s.clone(), o.clone(), component_name.to_owned()),
            rec
            )
    }

    pub fn set_scene_camera(
        &mut self,
        data : &mut Data<S>,
        ) -> operation::Change<S::Id>
    {
        println!("control remove sel");

        let sid = if let Some(ref s) = self.context.scene {
            s.clone()
        }
        else {
            return operation::Change::None;
        };

        let current = {
            let s = data.get_scene(sid.clone()).unwrap();
            s.get_camera_obj()
        };

        let o = self.get_selected_object();
        println!("control set camera");

        let vs = Vec::new();
        return self.request_operation(
            vs,
            operation::OperationData::SetSceneCamera(sid,current, o.clone()),
            data
            );
    }

}

