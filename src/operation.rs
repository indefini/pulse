use std::any::{Any};//, AnyRefExt};
use std::sync::{RwLock, Arc};
use std::marker::PhantomData;
use uuid;

use dormin::object;
use dormin::property;
use dormin::property::PropertyWrite;
use ui::PropertyUser;
use dormin::component::CompData;
use ui::RefMut;
use data::{ToId, SceneT};
use ui;

use dragger;

pub trait OperationReceiver {
    type Id;
    fn getP(&mut self, id : Self::Id) -> Option<&mut PropertyWrite>
    {
        println!("TODO {}, {}", file!(), line!());
        None
    }

    fn getP_copy(&mut self, id : Self::Id) -> Option<Box<PropertyWrite>>
    {
        println!("TODO {}, {}", file!(), line!());
        None
    }
}

trait OperationTrait
{
    type Id;
    fn apply(&self, rec : &mut OperationReceiver<Id=Self::Id>) -> Change<Self::Id>;
    fn undo(&self, rec : &mut OperationReceiver<Id=Self::Id>) -> Change<Self::Id>;
}

pub enum OperationData<Scene : SceneT>
{
    VecAdd(usize),
    VecDel(usize, Box<Any>),
    Vector(Vec<Box<Any>>, Vec<Box<Any>>),
    SceneAddObjects(Scene, Vec<Scene::Id>, Vec<Scene::Object>), //scene, parent, objects
    SceneRemoveObjects(Scene, Vec<Scene::Id>, Vec<Scene::Object>),
    SetSceneCamera(Scene, Option<Scene::Object>, Option<Scene::Object>),
    //AddComponent(uuid::Uuid, uuid::Uuid) //object id, component id?
    AddComponent(Scene::Object, Box<CompData>),
    OldNewVec(Vec<Box<Any>>, Box<Any>),

    //To check
    //ToNone(Box<Any>),
    //ToSome,
    //Function(fn(Vec<Object>, Box<Any>), Box<Any>),
}


pub struct Operation<S: SceneT>
{
    pub objects : Vec<Box<S::Object>>,
    pub name : Vec<String>,
    pub change : OperationData<S>
    //pub old : Box<Any>,
    //pub new : Box<Any>,
}

//TODO check if use this or erase
/*
pub enum OperationActor{
    Scene(uuid::Uuid),
    Object(uuid::Uuid),
    Objects(Vec<uuid::Uuid>),
    Ref(RefMut<PropertyWrite>),
    //PropertyWrite(&PropertyWrite),
}

pub struct OperationNew
{
    pub actor : OperationActor,
    pub name : String,
    pub change : OperationDataOld
    //pub old : Box<Any>,
    //pub new : Box<Any>,
}
*/

pub struct OldNew<S:SceneT>
{
    pub object : RefMut<PropertyUser>,
    pub name : String,
    pub old : Box<Any>,
    pub new : Box<Any>,
    phantom : PhantomData<S>
}

impl<S:SceneT> OldNew<S>
{
    pub fn new(
        object : RefMut<PropertyUser>,
        name : String,
        old : Box<Any>,
        new : Box<Any>
        ) -> OldNew<S>
    {
        OldNew{
            object : object,
            name : name,
            old : old,
            new : new,
            phantom : PhantomData
        }
    }

}

impl<S:SceneT> OperationTrait for OldNew<S>
{
    type Id = S::Id;
    //fn apply(&self ) -> Change
    fn apply(&self, rec : &mut OperationReceiver<Id=Self::Id>) -> Change<Self::Id>
    {
        println!("NEW TEST operation set property hier {:?}", self.name);
        match self.object {
            RefMut::Arc(ref a) => {
                a.write().unwrap().test_set_property_hier(self.name.as_ref(), &*self.new);
            },
            RefMut::Cell(ref c) => { 
                c.borrow_mut().test_set_property_hier(self.name.as_ref(), &*self.new);
            }
        }

        Change::Property(self.object.clone(), self.name.clone())
    }

    //fn undo(&self) -> Change
    fn undo(&self, rec : &mut OperationReceiver<Id=Self::Id>) -> Change<Self::Id>
    {
        match self.object {
            RefMut::Arc(ref a) => {
                a.write().unwrap().test_set_property_hier(self.name.as_ref(), &*self.old);
            },
            RefMut::Cell(ref c) => { 
                c.borrow_mut().test_set_property_hier(self.name.as_ref(), &*self.old);
            }
        }

        Change::Property(self.object.clone(), self.name.clone())
    }
}

pub struct ToNone{
    pub object : RefMut<PropertyUser>,
    pub name : String,
    pub old : Box<Any>,
}

impl ToNone
{
    pub fn new(
        object : RefMut<PropertyUser>,
        name : String,
        old : Box<Any>
        ) -> ToNone
    {
        ToNone{
            object : object,
            name : name,
            old : old,
        }
    }
}

impl OperationTrait for ToNone
{
    type Id = ui::def::Id;
    //fn apply(&self) -> Change
    fn apply(&self, rec : &mut OperationReceiver<Id=Self::Id>) -> Change<Self::Id>
    {
        println!("TO NONE operation set property hier {:?}", self.name);
        match self.object {
            RefMut::Arc(ref a) => {
                a.write().unwrap().set_property_hier(self.name.as_ref(), property::WriteValue::None);
            },
            RefMut::Cell(ref c) => { 
                c.borrow_mut().set_property_hier(self.name.as_ref(), property::WriteValue::None);
            }
        }

        Change::Property(self.object.clone(), self.name.clone())
    }

    //fn undo(&self) -> Change
    fn undo(&self, rec : &mut OperationReceiver<Id=Self::Id>) -> Change<Self::Id>
    {
        match self.object {
            RefMut::Arc(ref a) => {
                a.write().unwrap().test_set_property_hier(self.name.as_ref(), &*self.old);
            },
            RefMut::Cell(ref c) => { 
                c.borrow_mut().test_set_property_hier(self.name.as_ref(), &*self.old);
            }
        }

        Change::Property(self.object.clone(), self.name.clone())
    }
}

pub struct ToSome{
    pub object : RefMut<PropertyUser>,
    pub name : String,
}

impl ToSome
{
    pub fn new(
        object : RefMut<PropertyUser>,
        name : String
        ) -> ToSome
    {
        ToSome{
            object : object,
            name : name,
        }
    }
}

impl OperationTrait for ToSome
{
    type Id = ui::def::Id;
    //fn apply(&self) -> Change
    fn apply(&self, rec : &mut OperationReceiver<Id=Self::Id>) -> Change<Self::Id>
    {
        println!("TO Some operation set property hier {:?}", self.name);
        match self.object {
            RefMut::Arc(ref a) => {
                a.write().unwrap().set_property_hier(self.name.as_ref(), property::WriteValue::Some);
            },
            RefMut::Cell(ref c) => { 
                c.borrow_mut().set_property_hier(self.name.as_ref(), property::WriteValue::Some);
            }
        }

        Change::Property(self.object.clone(), self.name.clone())
    }

    //fn undo(&self) -> Change
    fn undo(&self, rec : &mut OperationReceiver<Id=Self::Id>) -> Change<Self::Id>
    {
        match self.object {
            RefMut::Arc(ref a) => {
                a.write().unwrap().set_property_hier(self.name.as_ref(), property::WriteValue::None);
            },
            RefMut::Cell(ref c) => { 
                c.borrow_mut().set_property_hier(self.name.as_ref(), property::WriteValue::None);
            }
        }

        Change::Property(self.object.clone(), self.name.clone())
    }
}

pub type ChangeOld = Change<uuid::Uuid>;

//#[derive(PartialEq)]
pub enum Change<Id>
{
    None,
    Property(RefMut<PropertyUser>, String),
    Objects(String, Vec<Id>),
    DirectChange(String),
    SceneAdd(Id, Vec<Id>, Vec<Id>),
    SceneRemove(Id, Vec<Id>, Vec<Id>),

    //check
    Scene(Id),
    ComponentChanged(Id, String),

    VecAdd(Vec<Id>, String, usize),
    VecDel(Vec<Id>, String, usize),

    DraggerOperation(dragger::Operation),
}

impl<S:SceneT> Operation<S>
{
    pub fn new(
        objects : Vec<Box<S::Object>>,
        name : Vec<String>,
        change : OperationData<S>
            )
        -> Operation<S>
    {

        /*
        fn fntest( objs : LinkedList<Arc<RwLock<object::Object>>>, val : Box<Any>)
        {
        }

        let test = OperationData::Function(fntest, box 5);
        */

        /*
        if let OperationData::OldNew(ref old,ref new) = change {
        match old.downcast_ref::<vec::Vec3>() {
            Some(v) => println!("old : {:?}", v),
            _ => {}
        }
        match new.downcast_ref::<vec::Vec3>() {
            Some(v) => println!("new : {:?}", v),
            _ => {}
        }
        }
        */

        //let change = OperationData::OldNew(old, new);
        Operation {
            objects : objects,
            name : name,
            change : change
        }
    }

}

impl<S:SceneT> OperationTrait for Operation<S>
//impl OperationTrait for Operation<ui::def::Scene>
{
    //type Id=ui::def::Id;
    type Id=S::Id;
    //fn apply(&self) -> Change
    fn apply(&self, rec : &mut OperationReceiver<Id=Self::Id>) -> Change<Self::Id>
    {
        match self.change {
            /*
            OperationData::ToNone(_) => {
                println!("to none, apply,  operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.set_property_hier(s.as_ref(), property::WriteValue::None);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            */
            OperationData::VecAdd(i) => {
                println!("vec add operation {:?}, {}", self.name,i);
                let s = join_string(&self.name);
                let mut ids = Vec::new();
                for o in &self.objects {
                    if let Some(mut p) = rec.getP_copy(o.to_id()) {
                        p.add_item(s.as_ref(), i, &String::from("empty"));
                    }
                    ids.push(o.to_id());
                }
                return Change::VecAdd(ids, s, i);
            },
            OperationData::VecDel(i,_) => {
                println!("vec del operation {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = Vec::new();
                for o in &self.objects {
                    if let Some(mut p) = rec.getP_copy(o.to_id()) {
                        p.del_item(s.as_ref(), i);
                    }
                    ids.push(o.to_id().clone());
                }
                return Change::VecDel(ids, s, i);
            },
            /*
            OperationData::ToSome => {
                println!("to some, apply,  operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.set_property_hier(s.as_ref(), property::WriteValue::Some);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            */
            OperationData::Vector(_,ref new) => {
                let mut i = 0;
                let s = join_string(&self.name);
                let sp = if !self.name.is_empty() && self.name.last().unwrap() == "*" {
                    let mut sp = self.name.clone();
                    sp.pop();
                    join_string(&sp)
                }
                else {
                    s.clone()
                };
                let mut ids = Vec::new();
                for o in &self.objects {
                    if let Some(mut p) = rec.getP_copy(o.to_id()) {
                    p.test_set_property_hier(
                        sp.as_str(),
                        &*new[i]);
                    }
                    i = i +1;
                    ids.push(o.to_id());
                }
                return Change::Objects(s, ids);
            },
            OperationData::SceneAddObjects(ref s, ref parents, ref obs)  => {
                //let mut sc = s.borrow_mut();
                //sc.add_objects(parents, obs);
                //return Change::SceneAdd(sc.id.clone(), parents.clone(), get_ids(obs));
                s.add_objects(parents, obs);
                return Change::SceneAdd(s.to_id(), parents.clone(), get_ids::<S>(obs));
            },
            OperationData::SceneRemoveObjects(ref s, ref parents, ref obs)  => {
                //let mut sc = s.borrow_mut();
                //sc.remove_objects(parents, obs);
                //return Change::SceneRemove(sc.id.clone(), parents.clone(), get_ids(obs));
                s.remove_objects(parents, obs);
                return Change::SceneRemove(s.to_id(), parents.clone(), get_ids::<S>(obs));
            },
            OperationData::SetSceneCamera(ref s, _, ref new)   => {
                println!("operation set camera");
                /*
                let sc = s.borrow();
                if let Some(ref c) = sc.camera {
                    if let Some(ref o) = *new {
                        println!("I set thhe camera !!!!!!!");
                        c.borrow_mut().object = o.clone();
                        c.borrow_mut().object_id = Some(o.read().unwrap().id.clone());
                        return Change::Scene(sc.id.clone());
                    }
                    else {
                        println!("dame 10");
                        c.borrow_mut().object_id = None;
                    }
                }
                else {
                    println!("dame 00");
                }
                */
                s.set_camera(new.clone());
                return Change::Scene(s.to_id());
            },
            OperationData::AddComponent(ref o, ref compo)  => {
                //TODO
                println!("TODO add component is not working of course, {}, {}", file!(), line!());
                /*
                let mut ob = o.write().unwrap();
                ob.add_comp_data(compo.clone());
                return Change::ComponentChanged(ob.id.clone(), compo.get_kind_string());
                */
            },
            _ => {}
        }

        Change::None
    }

    //fn undo(&self) -> Change
    fn undo(&self, rec : &mut OperationReceiver<Id=Self::Id>) -> Change<Self::Id>
    {
        match self.change {
            /*
            OperationData::ToNone(ref old) => {
                println!("to none, undo, operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.test_set_property_hier(s.as_ref(), &**old);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            OperationData::ToSome => {
                println!("to some, undo,  operation set property hier {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = LinkedList::new();
                for o in &self.objects {
                    let mut ob = o.write().unwrap();
                    ob.set_property_hier(s.as_ref(), property::WriteValue::None);
                    ids.push_back(ob.id.clone());
                }
                return Change::Objects(s, ids);
            },
            */
            OperationData::VecAdd(i) => {
                println!("vec add operation undo {:?}, {}", self.name, i);
                let s = join_string(&self.name);
                let mut ids = Vec::new();
                for o in &self.objects {
                    if let Some(mut p) = rec.getP_copy(o.to_id()) {
                        p.del_item(s.as_ref(), i);
                    }
                    ids.push(o.to_id());
                }
                return Change::VecDel(ids, s, i);
            },
            OperationData::VecDel(i,ref value) => {
                println!("vec del operation undo {:?}", self.name);
                let s = join_string(&self.name);
                let mut ids = Vec::new();
                for o in &self.objects {
                    //let mut ob = o.write().unwrap();
                    if let Some(mut p) = rec.getP_copy(o.to_id()) {
                        p.add_item(s.as_ref(), i, &**value);
                    }
                    ids.push(o.to_id());
                }
                //return Change::Objects(s, ids);
                return Change::VecAdd(ids, s, i);
            },
            OperationData::Vector(ref old,_) => {
                let mut i = 0;
                let s = join_string(&self.name);
                let sp = if !self.name.is_empty() && self.name.last().unwrap() == "*" {
                    let mut sp = self.name.clone();
                    sp.pop();
                    join_string(&sp)
                }
                else {
                    s.clone()
                };
                let mut ids = Vec::new();
                for o in &self.objects {
                    //let mut ob = o.write().unwrap();
                    if let Some(mut p) = rec.getP_copy(o.to_id()) {
                        p.test_set_property_hier(
                            sp.as_str(),
                            &*old[i]);
                    }
                    i = i +1;
                    ids.push(o.to_id());
                }
                return Change::Objects(s, ids);
            },
            OperationData::SceneAddObjects(ref s, ref parents, ref obs)  => {
                println!("undo scene add objects !!!");
                //let mut sc = s.borrow_mut();
                //sc.remove_objects(parents, obs);
                //return Change::SceneRemove(sc.id.clone(), parents.clone(), get_ids(obs));
                s.remove_objects(parents, obs);
                return Change::SceneRemove(s.to_id(), parents.clone(), get_ids::<S>(obs));
            },
            OperationData::SceneRemoveObjects(ref s, ref parents, ref obs)  => {
                println!("undo scene remove objects !!!");
                //let mut sc = s.borrow_mut();
                //sc.add_objects(parents, obs);
                //return Change::SceneAdd(sc.id.clone(), parents.clone(), get_ids(obs));
                s.remove_objects(parents, obs);
                return Change::SceneRemove(s.to_id(), parents.clone(), get_ids::<S>(obs));
            },
            OperationData::SetSceneCamera(ref s, ref old, _)   => {
                /*
                let sc = s.borrow();
                if let Some(ref c) = sc.camera {
                    if let Some(ref o) = *old {
                        c.borrow_mut().object = o.clone();
                        c.borrow_mut().object_id = Some(o.read().unwrap().id.clone());
                        return Change::Scene(sc.id.clone());
                    }
                    else {
                        c.borrow_mut().object_id = None
                    }
                }
                */
                s.set_camera(old.clone());
                return Change::Scene(s.to_id());
            },
            OperationData::AddComponent(ref o, ref compo)  => {
                println!("TODO add component is not working of course, {}, {}", file!(), line!());
                //let mut ob = o.write().unwrap();
                //ob.remove_comp_data(compo.clone());
                //return Change::ComponentChanged(ob.id.clone(), compo.get_kind_string());
            },
            _ => {}
        }

        Change::None
    }
}

trait AnyClone: Any + Clone {
}
impl<T: Any + Clone> AnyClone for T {}

pub struct OperationManager<Id>
{
    //pub undo : Vec<Operation>,
    //pub redo : Vec<Operation>,
    pub undo : Vec<Box<OperationTrait<Id=Id>+'static>>,
    pub redo : Vec<Box<OperationTrait<Id=Id>+'static>>,
    phantom : PhantomData<Id>
}

impl<Id> OperationManager<Id>
{
    pub fn new(
        ) -> OperationManager<Id>
    {
        OperationManager {
            undo : Vec::new(),
            redo : Vec::new(),
            phantom : PhantomData
        }
    }

    /*
    pub fn add(&mut self, op : OperationOld, rec : &mut OperationRec) -> ChangeOld
    {
        let change = op.apply(rec);
        self.add_undo(box op);
        self.redo.clear();

        change
    }
    */

    pub fn add_with_trait(&mut self, op : Box<OperationTrait<Id=Id>>, rec : &mut OperationReceiver<Id=Id>) -> Change<Id>
    {
        let change = op.apply(rec);
        self.add_undo(op);
        self.redo.clear();

        change
    }

    pub fn add_with_trait2(&mut self, op : Box<OperationTrait<Id=Id>>)
    {
        self.add_undo(op);
        self.redo.clear();
    }

    fn add_undo(&mut self, op : Box<OperationTrait<Id=Id>+'static>)
    {
        self.undo.push(op);
    }

    fn add_redo(&mut self, op : Box<OperationTrait<Id=Id>+'static>)
    {
        self.redo.push(op);
    }


    fn pop_undo(&mut self) -> Option<Box<OperationTrait<Id=Id>+'static>>
    {
        self.undo.pop()
    }

    fn pop_redo(&mut self) -> Option<Box<OperationTrait<Id=Id>+'static>>
    {
        self.redo.pop()
    }

    pub fn undo(&mut self, rec : &mut OperationReceiver<Id=Id>) -> Change<Id>
    {
        let op = match self.pop_undo() {
            Some(o) => o,
            None => {
                println!("nothing to undo");
                return Change::None;
            }
        };

        let change = op.undo(rec);

        self.add_redo(op);

        return change;
    }

    pub fn redo(&mut self, rec : &mut OperationReceiver<Id=Id>) -> Change<Id>
    {
        let op = match self.pop_redo() {
            Some(o) => o,
            None => {
                println!("nothing to redo");
                return Change::None;
            }
        };

        let change = op.apply(rec);

        self.add_undo(op);
        return change;
    }


}

//TODO remove
fn join_string(path : &[String]) -> String
{
    let mut s = String::new();
    let mut first = true;
    for v in path {
        if !first {
            s.push('/');
        }
        s.push_str(v.as_ref());
        first = false;
    }

    s
}

fn get_ids_old(obs : &[Arc<RwLock<object::Object>>]) -> Vec<uuid::Uuid>
{
    let mut list = Vec::new();
    for o in obs {
        list.push(o.read().unwrap().id.clone());
    }

    list
}

fn get_ids<S:SceneT>(obs : &[S::Object]) -> Vec<S::Id>
{
    let mut list = Vec::new();
    for o in obs {
        list.push(o.to_id());
    }

    list
}
