use dormin::vec;
use dormin::transform;
use dormin::geometry;
use dormin::intersection;
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

pub struct TranslationMove
{
    translation_start : vec::Vec3,
    constraint : vec::Vec3,
    repere : Repere,
    ori : vec::Quat
}


impl TranslationMove {
    pub fn new(
        start : vec::Vec3,
        constraint : vec::Vec3,
        repere : Repere,
        ori : vec::Quat
        ) -> TranslationMove
    {
        TranslationMove {
            translation_start : start,
            constraint : constraint,
            repere : repere,
            ori : ori
        }
    }

    fn global(
        &self,
        camera : &camera2::CameraTransform,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2) -> Option<Operation>
    {
        let mut p = geometry::Plane {
            point : self.translation_start,
            normal : camera.transform.orientation.rotate_vec3(
                &vec::Vec3::new(0f64,0f64,-1f64))
        };

        let constraint = self.constraint;

        if constraint != vec::Vec3::new(1f64,1f64,1f64) {
            if constraint.z == 1f64 {
                p.normal.z = 0f64;
            }
            if constraint.y == 1f64 {
                p.normal.y = 0f64;
            }
            if constraint.x == 1f64 {
                p.normal.x = 0f64;
            }
        }

        p.normal = p.normal.normalized();

        let rstart = camera.ray_from_screen(mouse_start.x, mouse_start.y, 1f64);
        let r = camera.ray_from_screen(mouse_end.x, mouse_end.y, 1f64);

        let irstart = intersection::intersection_ray_plane(&rstart, &p);
        let ir = intersection::intersection_ray_plane(&r, &p);

        if ir.hit && irstart.hit {
            let mut translation = ir.position - irstart.position;
            translation = translation * constraint;

            //let pos = self.translation_start + translation;
            //return Some(Operation::Translation(pos));
            return Some(Operation::Translation(translation));
        }
        else {
            return None;
        }
    }

    fn local(
        &self,
        camera : &camera2::CameraTransform,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2) -> Option<Operation>
    {
        let constraint = self.constraint;
        let ori = self.ori;

        let camup = camera.transform.orientation.rotate_vec3(&vec::Vec3::new(0f64,1f64,0f64));

        //printf("dragger ori : %f, %f, %f %f \n ", c->dragger_ori.x, c->dragger_ori.y, c->dragger_ori.z, c->dragger_ori.w);
        let ca = ori.rotate_vec3(&constraint);
        let cax = ori.rotate_vec3(&vec::Vec3::new(constraint.x,0f64,0f64));
        let cay = ori.rotate_vec3(&vec::Vec3::new(0f64,constraint.y,0f64));
        let caz = ori.rotate_vec3(&vec::Vec3::new(0f64,0f64,constraint.z));
        //printf("ca %f, %f, %f \n", ca.x, ca.y, ca.z);
        let mut n = camup ^ ca;
        if constraint == vec::Vec3::new(1f64,1f64,0f64) {
            n = ori.rotate_vec3(&vec::Vec3::new(0f64,0f64,1f64));
        }
        else if constraint == vec::Vec3::new(1f64,0f64,1f64) {
            n = ori.rotate_vec3(&vec::Vec3::new(0f64,1f64,0f64));
        }
        else if constraint == vec::Vec3::new(0f64,1f64,1f64) {
            n = ori.rotate_vec3(&vec::Vec3::new(1f64,0f64,0f64));
        }

        n.normalize();
        let mut p = geometry::Plane{ point : self.translation_start, normal : n };
        //printf("n %f, %f, %f \n", n.x, n.y, n.z);

        if constraint == vec::Vec3::new(0f64,1f64,0f64) {//TODO change this by checking the angle between camup and ca
            let camright = camera.transform.orientation.rotate_vec3(&vec::Vec3::new(1f64,0f64,0f64));
            p.normal = camright ^ ca;
        }

        let rstart = camera.ray_from_screen(mouse_start.x, mouse_start.y, 1f64);
        let r = camera.ray_from_screen(mouse_end.x, mouse_end.y, 1f64);

        let ir = intersection::intersection_ray_plane(&r, &p);
        let irstart = intersection::intersection_ray_plane(&rstart, &p);

        if ir.hit && irstart.hit {
            let mut translation = ir.position - irstart.position;
            //printf("translation %f, %f, %f \n", translation.x, translation.y, translation.z);
            if constraint == vec::Vec3::new(1f64,0f64,0f64) ||
               constraint == vec::Vec3::new(0f64,1f64,0f64) ||
               constraint == vec::Vec3::new(0f64,0f64,1f64) {
                   let dot = ca.dot(&translation);
                   translation = ca * dot;
            }

            //let pos = self.translation_start + translation;
            //return Some(Operation::Translation(pos));
            return Some(Operation::Translation(translation));
        }
        else {
            return None;
        }
    }

}

impl DraggerMouse for TranslationMove {

    fn mouse_move(
        &self,
        camera : &camera2::CameraTransform,
        mouse_start : vec::Vec2,
        mouse_end : vec::Vec2) -> Option<Operation>
    {
        match self.repere {
            Repere::Global => {
                return self.global(camera, mouse_start, mouse_end);
            },
            Repere::Local => {
                return self.local(camera, mouse_start, mouse_end);
            },
        }
    }
}

pub fn create_dragger_translation_group() -> DraggerGroup
{
    let red = vec::Vec4::new(1.0f64,0.247f64,0.188f64,0.5f64);
    let green = vec::Vec4::new(0.2117f64,0.949f64,0.4156f64,0.5f64);
    let blue = vec::Vec4::new(0f64,0.4745f64,1f64,0.5f64);
    let mesh = MESH_ARROW_NAME;
    let mesh_plane = MESH_PLANE_NAME;

    let dragger_x = Dragger::new(
        create_dragger("dragger_x", mesh, red),
        vec::Vec3::new(1f64,0f64,0f64),
        transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(vec::Vec3::new(0f64,1f64,0f64), 90f64)),
        Kind::Translate,
        red,
        Collision::MeshAABox
        );

    let dragger_y = Dragger::new(
        create_dragger("dragger_y", mesh, green),
        vec::Vec3::new(0f64,1f64,0f64),
        transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(vec::Vec3::new(1f64,0f64,0f64), -90f64)),
        Kind::Translate,
        green,
        Collision::MeshAABox
        );

    let dragger_z = Dragger::new(
        create_dragger("dragger_z", mesh, blue),
        vec::Vec3::new(0f64,0f64,1f64),
        transform::Orientation::Quat(vec::Quat::identity()),
        Kind::Translate,
        blue,
        Collision::MeshAABox
        );

    let dragger_xy = Dragger::new(
        create_dragger("dragger_xy", mesh_plane, red),
        vec::Vec3::new(1f64,1f64,0f64),
        transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(
                vec::Vec3::new(0f64,1f64,0f64), 90f64)),
        Kind::Translate,
        red,
        Collision::MeshAABox
        );

    let dragger_xz = Dragger::new(
        create_dragger("dragger_xz", mesh_plane, green),
        vec::Vec3::new(1f64,0f64,1f64),
        transform::Orientation::Quat(
            vec::Quat::new_axis_angle_deg(vec::Vec3::new(0f64,0f64,1f64), -90f64)),
        Kind::Translate,
        green,
        Collision::MeshAABox
        );

    let dragger_yz = Dragger::new(
        create_dragger("dragger_yz", mesh_plane, blue),
        vec::Vec3::new(0f64,1f64,1f64),
        //transform::Orientation::Quat(vec::Quat::new_axis_angle_deg(vec::Vec3::new(1f64,0f64,0f64), 90f64)),
        transform::Orientation::Quat(vec::Quat::identity()),
        Kind::Translate,
        blue,
        Collision::MeshAABox
        );

    let mut group = Vec::with_capacity(6);

    group.push(dragger_x);
    group.push(dragger_y);
    group.push(dragger_z);

    group.push(dragger_xy);
    group.push(dragger_xz);
    group.push(dragger_yz);

    return group;
}

pub static MESH_ARROW_NAME: &'static str = "model/dragger_arrow.mesh";
pub static MESH_PLANE_NAME: &'static str = "model/dragger_plane.mesh";

pub static MESH_ARROW: &'static [u8] = include_bytes!("../../../avion/model/dragger_arrow.mesh");
pub static MESH_PLANE: &'static [u8] = include_bytes!("../../../avion/model/dragger_plane.mesh");

