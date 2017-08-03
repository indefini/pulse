use std::any::{Any};//, AnyRefExt};
use std::marker::PhantomData;

use dormin::property;
use data::{Data, ToId, SceneT};

use dragger;

pub trait OperationTrait
{
    type Scene : SceneT;
    type Id;
    fn apply(&self, rec : &mut Data<Self::Scene>) -> Change<Self::Id>;
    fn undo(&self, rec : &mut Data<Self::Scene>) -> Change<Self::Id>;
}

pub enum OperationData<Scene : SceneT>
{
    VecAdd(usize),
    VecDel(usize, Box<Any>),
    Vector(Vec<Box<Any>>, Vec<Box<Any>>),
    SceneAddObjects(Scene::Id, Vec<Option<Scene::Id>>, Vec<Scene::Object>), //scene, parent, objects
    SceneRemoveObjects(Scene::Id, Vec<Option<Scene::Id>>, Vec<Scene::Object>),
    SetSceneCamera(Scene::Id, Option<Scene::Object>, Option<Scene::Object>),
    //TODO
    //AddComponent(Scene::Object, Box<CompData>),
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
    pub object_id : S::Id,
    pub name : String,
    pub old : Box<Any>,
    pub new : Box<Any>,
    phantom : PhantomData<S>
}

impl<S:SceneT> OldNew<S>
{
    pub fn new(
        object_id : S::Id,
        name : String,
        old : Box<Any>,
        new : Box<Any>
        ) -> OldNew<S>
    {
        OldNew{
            object_id : object_id,
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
    type Scene = S;
    fn apply(&self, rec : &mut Data<S>) -> Change<Self::Id>
    {
        println!("NEW TEST operation set property hier {:?}", self.name);

        if let Some((mut p, s)) = rec.get_property_write_copy(self.object_id.clone(), &self.name) {
            p.test_set_property_hier(s.as_ref(), &*self.new);
        }
        //TODO not object id but scene id
        else if let Some((mut p, s)) = rec.get_property_write(self.object_id.clone(), self.object_id.clone(), &self.name) {
            p.test_set_property_hier(s.as_ref(), &*self.new);
        }

        Change::PropertyId(self.object_id.clone(), self.name.clone())
    }

    fn undo(&self, rec : &mut Data<S>) -> Change<Self::Id>
    {
        if let Some((mut p, s)) = rec.get_property_write_copy(self.object_id.clone(), &self.name) {
            p.test_set_property_hier(s.as_ref(), &*self.old);
        }
        else if let Some((mut p, s)) = rec.get_property_write(self.object_id.clone(), self.object_id.clone(), &self.name) {
            p.test_set_property_hier(s.as_ref(), &*self.old);
        }
        else {
            panic!("could not get the property write in undo oldnew");
        }

        Change::PropertyId(self.object_id.clone(), self.name.clone())
    }
}

pub struct ToNone<S:SceneT>{
    pub object_id : S::Id,
    pub name : String,
    pub old : Box<Any>,
}

impl<S:SceneT> ToNone<S>
{
    pub fn new(
        object_id : S::Id,
        name : String,
        old : Box<Any>
        ) -> ToNone<S>
    {
        ToNone{
            object_id : object_id,
            name : name,
            old : old,
        }
    }
}

impl<S:SceneT> OperationTrait for ToNone<S>
{
    type Id = S::Id;
    type Scene = S;
    fn apply(&self, rec : &mut Data<S>) -> Change<Self::Id>
    {
        println!("TO NONE operation set property hier {:?}", self.name);
        if let Some((mut p, s)) = rec.get_property_write_copy(self.object_id.clone(), &self.name) {
            p.set_property_hier(s.as_ref(), property::WriteValue::None);
        }

        Change::PropertyId(self.object_id.clone(), self.name.clone())
    }

    fn undo(&self, rec : &mut Data<S>) -> Change<Self::Id>
    {
        if let Some((mut p, s)) = rec.get_property_write_copy(self.object_id.clone(), &self.name) {
            p.test_set_property_hier(s.as_ref(), &*self.old);
        }

        Change::PropertyId(self.object_id.clone(), self.name.clone())
    }
}

pub struct ToSome<S:SceneT>{
    pub object_id : S::Id,
    pub name : String,
}

impl<S:SceneT> ToSome<S>
{
    pub fn new(
        object_id : S::Id,
        name : String
        ) -> ToSome<S>
    {
        ToSome{
            object_id : object_id,
            name : name,
        }
    }
}

impl<S:SceneT> OperationTrait for ToSome<S>
{
    type Id = S::Id;
    type Scene = S;
    fn apply(&self, rec : &mut Data<S>) -> Change<Self::Id>
    {
        println!("TO Some operation set property hier {:?}", self.name);
        if let Some((mut p, s)) = rec.get_property_write_copy(self.object_id.clone(), &self.name) {
            p.set_property_hier(s.as_ref(), property::WriteValue::Some);
        }

        Change::PropertyId(self.object_id.clone(), self.name.clone())
    }

    fn undo(&self, rec : &mut Data<S>) -> Change<Self::Id>
    {
        if let Some((mut p, s)) = rec.get_property_write_copy(self.object_id.clone(), &self.name) {
            p.set_property_hier(s.as_ref(), property::WriteValue::None);
        }

        Change::PropertyId(self.object_id.clone(), self.name.clone())
    }
}

//#[derive(PartialEq)]
pub enum Change<Id>
{
    None,
    PropertyId(Id, String),
    Objects(String, Vec<Id>),
    DirectChange(String),
    SceneAdd(Id, Vec<Option<Id>>, Vec<Id>),
    SceneRemove(Id, Vec<Option<Id>>, Vec<Id>),

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
{
    type Id=S::Id;
    type Scene=S;
    fn apply(&self, rec : &mut Data<S>) -> Change<Self::Id>
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
                    if let Some((p, news)) = rec.get_property_write(o.to_id(),o.to_id(),s.as_ref()) {
                        p.add_item(news.as_ref(), i, &String::from("empty"));
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
                    println!("TODO check this 's' and 'ss' variable");
                    if let Some((mut p, ss)) = rec.get_property_write_copy(o.to_id(), &s) {
                        p.del_item(ss.as_ref(), i);
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
                    //println!("please take the object with id '{:?}', and set the property '{}' to value {:?}",o.to_id(), sp, new[i]);
                    //println!("please take the object, and set the property '{}' to value {:?}", sp, new[i]);
                    if let Some((p, news)) = rec.get_property_write(o.to_id(),o.to_id(),sp.as_ref()) {
                        p.test_set_property_hier(
                            news.as_str(),
                            &*new[i]);
                    }

                    if let Some((mut p, news)) = rec.get_property_write_copy(o.to_id(),sp.as_ref()) {
                        p.test_set_property_hier(
                            news.as_str(),
                            &*new[i]);
                    }
                    else {
                        println!("ERR : !!! could not find property : {} !!!",sp);
                    }
                    i = i +1;
                    ids.push(o.to_id());
                }
                return Change::Objects(s, ids);
            },
            OperationData::SceneAddObjects(ref s, ref parents, ref obs)  => {
                rec.add_objects(s.clone(), parents, obs);
                return Change::SceneAdd(s.clone(), parents.clone(), get_ids::<S>(obs));
            },
            OperationData::SceneRemoveObjects(ref s, ref parents, ref obs)  => {
                rec.remove_objects(s.clone(), parents, obs);
                return Change::SceneRemove(s.clone(), parents.clone(), get_ids::<S>(obs));
            },
            OperationData::SetSceneCamera(ref s, _, ref new)   => {
                println!("operation set camera");
                rec.set_camera(s.clone(), new.clone());
                return Change::Scene(s.clone());
            },
                /*
            OperationData::AddComponent(ref o, ref compo)  => {
                //TODO
                println!("TODO add component is not working of course, {}, {}", file!(), line!());
                //let mut ob = o.write().unwrap();
                //ob.add_comp_data(compo.clone());
                //return Change::ComponentChanged(ob.id.clone(), compo.get_kind_string());
            },
                */
            _ => {
                unimplemented!();
            }
        }

        Change::None
    }

    fn undo(&self, rec : &mut Data<S>) -> Change<Self::Id>
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
                    if let Some((mut p, ss)) = rec.get_property_write_copy(o.to_id(), &s) {
                        p.del_item(ss.as_ref(), i);
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
                    if let Some((mut p, ss)) = rec.get_property_write_copy(o.to_id(), &s) {
                        p.add_item(ss.as_ref(), i, &**value);
                    }
                    ids.push(o.to_id());
                }
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
                    if let Some((p, news)) = rec.get_property_write(o.to_id(),o.to_id(),sp.as_ref()) {
                        p.test_set_property_hier(
                            news.as_str(),
                            &*old[i]);
                    }

                    if let Some((mut p, news)) = rec.get_property_write_copy(o.to_id(),sp.as_ref()) {
                        p.test_set_property_hier(
                            news.as_str(),
                            &*old[i]);
                    }


                    i = i +1;
                    ids.push(o.to_id());
                }
                return Change::Objects(s, ids);
            },
            OperationData::SceneAddObjects(ref s, ref parents, ref obs)  => {
                println!("undo scene add objects !!!");
                rec.remove_objects(s.clone(), parents, obs);
                return Change::SceneRemove(s.clone(), parents.clone(), get_ids::<S>(obs));
            },
            OperationData::SceneRemoveObjects(ref s, ref parents, ref obs)  => {
                println!("undo scene remove objects !!!");
                rec.add_objects(s.clone(), parents, obs);
                return Change::SceneAdd(s.clone(), parents.clone(), get_ids::<S>(obs));
            },
            OperationData::SetSceneCamera(ref s, ref old, _)   => {
                rec.set_camera(s.clone(), old.clone());
                return Change::Scene(s.clone());
            },
            /*
            OperationData::AddComponent(ref o, ref compo)  => {
                println!("TODO add component is not working of course, {}, {}", file!(), line!());
                //let mut ob = o.write().unwrap();
                //ob.remove_comp_data(compo.clone());
                //return Change::ComponentChanged(ob.id.clone(), compo.get_kind_string());
            },
            */
            _ => {
                unimplemented!();
            }
        }

        Change::None
    }
}

trait AnyClone: Any + Clone {
}
impl<T: Any + Clone> AnyClone for T {}

pub struct OperationManager<S:SceneT>
{
    undo : Vec<Box<OperationTrait<Id=S::Id,Scene=S>+'static>>,
    redo : Vec<Box<OperationTrait<Id=S::Id,Scene=S>+'static>>,
    phantom : PhantomData<S>
}

impl<S:SceneT> OperationManager<S>
{
    pub fn new(
        ) -> OperationManager<S>
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

    //pub fn add_with_trait(&mut self, op : Box<OperationTrait<Id=S::Id,Scene=S>>, rec : &mut OperationReceiver<Scene=S>) -> Change<S::Id>
    pub fn add_with_trait(
        &mut self,
        op : Box<OperationTrait<Id=S::Id,Scene=S>>,
        rec : &mut Data<S>) -> Change<S::Id>
    {
        let change = op.apply(rec);
        self.add_undo(op);
        self.redo.clear();

        change
    }

    pub fn add_with_trait2(&mut self, op : Box<OperationTrait<Id=S::Id,Scene=S>>)
    {
        self.add_undo(op);
        self.redo.clear();
    }

    fn add_undo(&mut self, op : Box<OperationTrait<Id=S::Id,Scene=S>+'static>)
    {
        self.undo.push(op);
    }

    fn add_redo(&mut self, op : Box<OperationTrait<Id=S::Id,Scene=S>+'static>)
    {
        self.redo.push(op);
    }


    fn pop_undo(&mut self) -> Option<Box<OperationTrait<Id=S::Id,Scene=S>+'static>>
    {
        self.undo.pop()
    }

    fn pop_redo(&mut self) -> Option<Box<OperationTrait<Id=S::Id,Scene=S>+'static>>
    {
        self.redo.pop()
    }

    pub fn undo(&mut self, rec : &mut Data<S>) -> Change<S::Id>
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

    pub fn redo(&mut self, rec : &mut Data<S>) -> Change<S::Id>
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

fn get_ids<S:SceneT>(obs : &[S::Object]) -> Vec<S::Id>
{
    let mut list = Vec::new();
    for o in obs {
        list.push(o.to_id());
    }

    list
}
