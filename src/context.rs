use data::{ToId, SceneT};

pub struct Context<S:SceneT>
{
    pub selected : Vec<S::Object>,
    pub scene : Option<S::Id>,
}

impl<S : Clone+SceneT> Context<S>
{
    pub fn new() -> Context<S>
    {
        Context {
            selected: Vec::new(),
            scene : None,
        }
    }

    pub fn set_scene(&mut self, scene : S::Id)
    {
        self.scene = Some(scene);
        self.selected.clear();
    }

    pub fn get_scene(&self) -> Option<S::Id>
    {
        self.scene.clone()
    }
}

impl<S:SceneT> Context<S>
{
    pub fn get_vec_selected_ids(&self) -> Vec<S::Id>
    {
        let mut v = Vec::with_capacity(self.selected.len());
        for o in &self.selected {
            v.push(o.to_id());
        }

        v
    }

    pub fn remove_objects_by_id(&mut self, ids : &[S::Id])
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

    pub fn has_object_with_id(&self, id : &S::Id) -> bool
    {
        for o in &self.selected {
            if *id == o.to_id() {
               return true;
            }
        }

        false
    }

    pub fn has_object(&self, ob : S::Object) -> bool
    {
        for o in &self.selected {
            if ob.to_id() == o.to_id() {
               return true;
            }
        }

        false
    }

    pub fn select_by_ob(&mut self, obs : &mut Vec<S::Object>)
    {
        self.selected.append(obs);
    }

    /*
    pub fn select_by_id(&mut self, ids : &mut Vec<S::Id>)
    {
        //TODO same as the code at the end of mouse_up, so factorize
        println!("TODO check: is this find by id ok? : control will try to find object by id, .................select is called ");

        //c.selected.clear();

        let scene = match self.scene {
            Some(ref s) => s.clone(),
            None => return
        };

        let mut obs = scene.find_objects_by_id(ids);
        self.selected.append(&mut obs);

        //for id in ids.iter() {
            //match scene.read().unwrap().find_object_by_id(id) {
                //Some(o) =>
                    //c.selected.push_back(o.clone()),
                //None => {}
            //};
        //}

    }
    */
}

