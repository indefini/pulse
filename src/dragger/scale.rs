use dormin::vec;
use dormin::transform;
use dormin::camera;
use dormin::camera2;

use dragger::manager::{
    Repere,
    Operation,
    DraggerMouse,
    DraggerGroup,
    Kind,
    Collision,
    Dragger,
    create_dragger,
};

pub struct ScaleOperation
{
    start : vec::Vec3,
    constraint : vec::Vec3,
    repere : Repere,
    ori : vec::Quat
}

impl ScaleOperation {

    pub fn new(
        start : vec::Vec3,
        constraint : vec::Vec3, 
        repere : Repere,
        ori : vec::Quat
        ) -> ScaleOperation
    {
        ScaleOperation {
            start : start,
            constraint : constraint,
            repere : repere,
            ori : ori
        }
    }

    fn local(
        &self,
        camera : &camera2::CameraTransform,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2) -> Option<Operation>
    {
        let ss = camera.world_to_screen(self.start);

        let sss = mouse_start - ss;
        let l1 = sss.length2();
        let sd = mouse_end - ss;
        let l2 = sd.length2();

        let mut fac = l2/l1;
        let dot = sss.dot(sd);
        if dot < 0f64 {
            fac *= -1f64;
        }

        let mut scale_factor = vec::Vec3::new(fac,fac,fac);
        if self.constraint.x == 0f64 {
            scale_factor.x = 1f64;
        }
        if self.constraint.y == 0f64 {
            scale_factor.y = 1f64;
        }
        if self.constraint.z == 0f64 {
            scale_factor.z = 1f64;
        }

        return Some(Operation::Scale(scale_factor));
    }
}


impl DraggerMouse for ScaleOperation {

    fn mouse_move(
        &self,
        camera : &camera2::CameraTransform,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2) -> Option<Operation>
    {
        return self.local(camera, mouse_start, mouse_end);
    }
}

pub fn create_scale_draggers() -> DraggerGroup
{
    let red = vec::Vec4::new(1.0f64,0.247f64,0.188f64,0.5f64);
    let green = vec::Vec4::new(0.2117f64,0.949f64,0.4156f64,0.5f64);
    let blue = vec::Vec4::new(0f64,0.4745f64,1f64,0.5f64);
    let mesh = MESH_SCALE_NAME;
    //let mesh_cube = "model/dragger_cube.mesh";

    let dragger_x = Dragger::new(
        create_dragger("scale_x", mesh, red),
        vec::Vec3::new(1f64,0f64,0f64),
        transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(vec::Vec3::new(0f64,1f64,0f64), 90f64)),
        Kind::Scale,
        red,
        Collision::MeshAABox
        );

    let dragger_y = Dragger::new(
        create_dragger("scale_y", mesh, green),
        vec::Vec3::new(0f64,1f64,0f64),
        transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(vec::Vec3::new(1f64,0f64,0f64), -90f64)), 
        Kind::Scale,
        green,
        Collision::MeshAABox
        );

    let dragger_z = Dragger::new(
        create_dragger("scale_z", mesh, blue),
        vec::Vec3::new(0f64,0f64,1f64),
        transform::Orientation::Quat(vec::Quat::identity()), 
        Kind::Scale,
        blue,
        Collision::MeshAABox
        );

    /*
    let dragger_xyz = Dragger::new(
        factory,
        "scale_xyz",
        mesh_cube,
        vec::Vec3::new(1f64,1f64,1f64),
        transform::Orientation::Quat(vec::Quat::identity()), 
        Kind::Scale,
        blue,
        Collision::MeshAABox
        );
        */

    let mut group = Vec::with_capacity(4);

    group.push(dragger_x);
    group.push(dragger_y);
    group.push(dragger_z);
    //group.push(Rc::new(RefCell::new(dragger_xyz)));

    return group;
}


pub static MESH_SCALE_NAME: &'static str = "model/dragger_scale.mesh";
pub static MESH_SCALE: &'static [u8] = include_bytes!("../../../avion/model/dragger_scale.mesh");

