use std::rc::{Rc};
use std::cell::{RefCell};
use std::any::Any;

use dormin::{vec, transform, component};
use dormin::property::PropertyGet;

use context;
use operation;
use data::{SceneT,ToId};

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

impl<S:SceneT+Clone+'static> State<S> {
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

    pub fn save_positions(&mut self)
    {
        let s = self.context.scene.as_ref().unwrap();
        self.saved_positions = 
            self.context.selected.iter().map(
                |o| s.get_position(o.clone())
                ).collect();
    }

    pub fn save_scales(&mut self)
    {
        let s = self.context.scene.as_ref().unwrap();
        self.saved_scales = self.context.selected.iter().map(|o| s.get_scale(o.clone())).collect();
    }

    pub fn save_oris(&mut self)
    {
        let s = self.context.scene.as_ref().unwrap();
        self.saved_oris = self.context.selected.iter().map(|o| s.get_orientation(o.clone())).collect();
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
        rec : &mut operation::OperationReceiver<Scene=S>
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

    pub fn undo(&mut self, rec : &mut operation::OperationReceiver<Scene=S>) -> operation::Change<S::Id>
    {
        self.op_mgr.undo(rec)
    }

    pub fn redo(&mut self, rec : &mut operation::OperationReceiver<Scene=S>) -> operation::Change<S::Id>
    {
        self.op_mgr.redo(rec)
    }

    pub fn remove_selected_objects(
        &mut self,
        rec : &mut operation::OperationReceiver<Scene=S>
        ) -> operation::Change<S::Id>
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
            let parent_id = s.get_parent(o.clone()).map_or(S::Id::default(), |x| x.to_id());
            parent.push(parent_id);
        }

        let vs = Vec::new();
        return self.request_operation(
            vs,
            operation::OperationData::SceneRemoveObjects(s.clone(), parent, vec),
            rec
            );

        //return operation::Change::SceneRemove(s.read().unwrap().id, vec);
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
        rec : &mut operation::OperationReceiver<Scene=S>
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
        translation : vec::Vec3) -> operation::Change<S::Id>
    {
        let s = self.context.scene.as_ref().unwrap();

        let sp = self.saved_positions.clone();

        for (i,o) in self.context.selected.iter().enumerate() {
            s.set_position(o.clone(), sp[i] + translation);
        }

        return operation::Change::DirectChange("position".to_owned());
    }

    pub fn request_scale(
        &mut self,
        scale : vec::Vec3) -> operation::Change<S::Id>
    {
        let s = self.context.scene.as_ref().unwrap();
        let sp = self.saved_scales.clone();

        for (i,o) in self.context.selected.iter().enumerate() {
            s.set_scale(o.clone(), sp[i] * scale);
        }

        return operation::Change::DirectChange("scale".to_owned());
    }

    pub fn request_rotation(
        &mut self,
        rotation : vec::Quat) -> operation::Change<S::Id>
    {
        let s = self.context.scene.as_ref().unwrap();
        let so = self.saved_oris.clone();

        for (i,o) in self.context.selected.iter().enumerate() {
            s.set_orientation(o.clone(), so[i] * transform::Orientation::new_with_quat(&rotation));
        }

        operation::Change::DirectChange("orientation/*".to_owned())
    }

    pub fn request_operation_property_old_new_dontcheckequal(
        &mut self,
        property : ui::RefMut<ui::PropertyUser>,
        name : &str,
        old : Box<Any>,
        new : Box<Any>,
        rec : &mut operation::OperationReceiver<Scene=S>
        ) -> operation::Change<S::Id>
    {
        let op : operation::OldNew<S> = operation::OldNew::new(
            property,
            String::from(name),
            old,
            new
            );

        let change = self.op_mgr.add_with_trait(box op, rec);
        change
    }

    pub fn request_operation_property_old_new<T : Any+PartialEq>(
        &mut self,
        property : ui::RefMut<ui::PropertyUser>,
        name : &str,
        old : Box<T>,
        new : Box<T>,
        rec : &mut operation::OperationReceiver<Scene=S>
        ) -> operation::Change<S::Id>
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

        let op : operation::OldNew<S> = operation::OldNew::new(
            property,
            String::from(name),
            old,
            new
            );

        let change = self.op_mgr.add_with_trait(box op, rec);
        change
    }

    pub fn request_direct_change_property(
        &mut self,
        property : &mut ui::PropertyUser,
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
        rec : &mut operation::OperationReceiver<Scene=S>
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
        rec : &mut operation::OperationReceiver<Scene=S>
        ) -> operation::Change<S::Id>
    {
        let s = match self.context.scene {
            Some(ref s) => s.clone(),
            None => return operation::Change::None
        };

        let mut vec = Vec::new();
        let mut parents = Vec::new();
        for o in &self.context.selected {
            println!("COPY is not working because of this TODO");
            //TODO vec.push(Arc::new(RwLock::new(factory.copy_object(&*ob))));
            let parent_id = s.get_parent(o.clone()).map_or(S::Id::default(), |x| x.to_id());

            parents.push(parent_id);
        }

        let vs = Vec::new();
        return self.request_operation(
            vs,
            operation::OperationData::SceneAddObjects(s.to_id(), parents, vec),
            rec
            );
    }

    pub fn add_component(
        &mut self,
        component_name : &str,
        rec : &mut operation::OperationReceiver<Scene=S>
        ) -> operation::Change<S::Id>
    {
        let o = if let Some(o) = self.context.selected.get(0) {
            o.clone()
        }
        else
        {
            return operation::Change::None;
        };

        let cp = if component_name == "MeshRender" {
            box component::CompData::MeshRender(component::mesh_render::MeshRender::with_names_only("model/skeletonmesh.mesh", "material/simple.mat"))
        }
        else {
            return operation::Change::None;
        };

        let vs = Vec::new();

        self.request_operation(
            vs,
            operation::OperationData::AddComponent(o.clone(), cp),
            rec
            )
    }

    pub fn set_scene_camera(
        &mut self,
        rec : &mut operation::OperationReceiver<Scene=S>
        ) -> operation::Change<S::Id>
    {
        println!("control remove sel");

        let s = match self.context.scene {
            Some(ref s) => s.clone(),
            None => return operation::Change::None
        };

        let current = s.get_camera_obj();

        let o = self.get_selected_object();
        println!("control set camera");

        let vs = Vec::new();
        return self.request_operation(
            vs,
            operation::OperationData::SetSceneCamera(s,current, o.clone()),
            rec
            );
    }

}

