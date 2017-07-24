use dormin::vec;
use dormin::transform;
use dormin::geometry;
use dormin::intersection;
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
    let mesh = "model/dragger_arrow.mesh";
    let mesh_plane = "model/dragger_plane.mesh";

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

//from mesh/dragger_arrow.mesh (made with xxd tool)
pub static MESH_ARROW : &'static [u8] = &[
  0x04, 0x00, 0x6d, 0x65, 0x73, 0x68, 0x0d, 0x00, 0x64, 0x72, 0x61, 0x67,
  0x67, 0x65, 0x72, 0x5f, 0x61, 0x72, 0x72, 0x6f, 0x77, 0x11, 0x00, 0xb9,
  0x1e, 0xa5, 0xb1, 0x42, 0x60, 0xe5, 0x3d, 0xfe, 0xff, 0xbf, 0x3f, 0x34,
  0x33, 0x93, 0xb1, 0x01, 0xc9, 0xda, 0xb0, 0x32, 0x33, 0xf3, 0x3f, 0x7d,
  0x31, 0xa2, 0x3d, 0x7e, 0x31, 0xa2, 0x3d, 0xfe, 0xff, 0xbf, 0x3f, 0x43,
  0x60, 0xe5, 0x3d, 0x6e, 0x62, 0xd7, 0xb1, 0xfe, 0xff, 0xbf, 0x3f, 0x7d,
  0x31, 0xa2, 0x3d, 0x7e, 0x31, 0xa2, 0xbd, 0xfe, 0xff, 0xbf, 0x3f, 0x2d,
  0xc6, 0x7a, 0xb2, 0x42, 0x60, 0xe5, 0xbd, 0xfe, 0xff, 0xbf, 0x3f, 0x80,
  0x31, 0xa2, 0xbd, 0x7c, 0x31, 0xa2, 0xbd, 0xfe, 0xff, 0xbf, 0x3f, 0x42,
  0x60, 0xe5, 0xbd, 0xc0, 0xd5, 0x23, 0xae, 0xfe, 0xff, 0xbf, 0x3f, 0x80,
  0x31, 0xa2, 0xbd, 0x7c, 0x31, 0xa2, 0x3d, 0xfe, 0xff, 0xbf, 0x3f, 0x00,
  0x00, 0x20, 0xb1, 0xcd, 0xcc, 0xcc, 0x3c, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x20, 0xb1, 0xcd, 0xcc, 0xcc, 0x3c, 0x00, 0x00, 0xc0, 0x3f, 0xcc,
  0xcc, 0xcc, 0x3c, 0x80, 0x49, 0x7e, 0xb0, 0x00, 0x00, 0x00, 0x00, 0xcc,
  0xcc, 0xcc, 0x3c, 0x80, 0x49, 0x7e, 0xb0, 0x00, 0x00, 0xc0, 0x3f, 0x79,
  0x18, 0x9b, 0xb1, 0xcd, 0xcc, 0xcc, 0xbc, 0x00, 0x00, 0x00, 0x00, 0x79,
  0x18, 0x9b, 0xb1, 0xcd, 0xcc, 0xcc, 0xbc, 0x00, 0x00, 0xc0, 0x3f, 0xce,
  0xcc, 0xcc, 0xbc, 0xdc, 0x0a, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0xce,
  0xcc, 0xcc, 0xbc, 0xdc, 0x0a, 0x00, 0x30, 0x00, 0x00, 0xc0, 0x3f, 0x1a,
  0x00, 0x07, 0x00, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02,
  0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x01, 0x00, 0x03,
  0x00, 0x06, 0x00, 0x01, 0x00, 0x07, 0x00, 0x05, 0x00, 0x01, 0x00, 0x06,
  0x00, 0x04, 0x00, 0x01, 0x00, 0x05, 0x00, 0x03, 0x00, 0x01, 0x00, 0x04,
  0x00, 0x00, 0x00, 0x02, 0x00, 0x08, 0x00, 0x02, 0x00, 0x03, 0x00, 0x08,
  0x00, 0x03, 0x00, 0x07, 0x00, 0x08, 0x00, 0x03, 0x00, 0x06, 0x00, 0x07,
  0x00, 0x03, 0x00, 0x04, 0x00, 0x06, 0x00, 0x04, 0x00, 0x05, 0x00, 0x06,
  0x00, 0x09, 0x00, 0x0a, 0x00, 0x0c, 0x00, 0x09, 0x00, 0x0c, 0x00, 0x0b,
  0x00, 0x0b, 0x00, 0x0c, 0x00, 0x0e, 0x00, 0x0b, 0x00, 0x0e, 0x00, 0x0d,
  0x00, 0x0c, 0x00, 0x0a, 0x00, 0x10, 0x00, 0x0c, 0x00, 0x10, 0x00, 0x0e,
  0x00, 0x0f, 0x00, 0x10, 0x00, 0x0a, 0x00, 0x0f, 0x00, 0x0a, 0x00, 0x09,
  0x00, 0x0d, 0x00, 0x0e, 0x00, 0x10, 0x00, 0x0d, 0x00, 0x10, 0x00, 0x0f,
  0x00, 0x09, 0x00, 0x0b, 0x00, 0x0d, 0x00, 0x09, 0x00, 0x0d, 0x00, 0x0f,
  0x00, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb4, 0xcd, 0x59, 0x3f, 0x0d,
  0x83, 0x06, 0xbf, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x80, 0x3f, 0x34, 0x03, 0x1a, 0x3f, 0x34, 0x03, 0x1a, 0x3f, 0x0d,
  0x83, 0x06, 0xbf, 0xb4, 0xcd, 0x59, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x0d,
  0x83, 0x06, 0xbf, 0x34, 0x03, 0x1a, 0x3f, 0x34, 0x03, 0x1a, 0xbf, 0x0d,
  0x83, 0x06, 0xbf, 0x00, 0x00, 0x00, 0x00, 0xb4, 0xcd, 0x59, 0xbf, 0x0d,
  0x83, 0x06, 0xbf, 0x34, 0x03, 0x1a, 0xbf, 0x34, 0x03, 0x1a, 0xbf, 0x0d,
  0x83, 0x06, 0xbf, 0xb4, 0xcd, 0x59, 0xbf, 0x00, 0x00, 0x00, 0x00, 0x0d,
  0x83, 0x06, 0xbf, 0x34, 0x03, 0x1a, 0xbf, 0x34, 0x03, 0x1a, 0x3f, 0x0d,
  0x83, 0x06, 0xbf, 0x00, 0x00, 0x00, 0x00, 0xa2, 0x05, 0x51, 0x3f, 0x28,
  0xcd, 0x13, 0xbf, 0x00, 0x00, 0x00, 0x00, 0xa2, 0x05, 0x51, 0x3f, 0x28,
  0xcd, 0x13, 0x3f, 0xa2, 0x05, 0x51, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x28,
  0xcd, 0x13, 0xbf, 0xa2, 0x05, 0x51, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x28,
  0xcd, 0x13, 0x3f, 0x00, 0x00, 0x00, 0x00, 0xa2, 0x05, 0x51, 0xbf, 0x28,
  0xcd, 0x13, 0xbf, 0x00, 0x00, 0x00, 0x00, 0xa2, 0x05, 0x51, 0xbf, 0x28,
  0xcd, 0x13, 0x3f, 0xa2, 0x05, 0x51, 0xbf, 0x00, 0x00, 0x00, 0x00, 0x28,
  0xcd, 0x13, 0xbf, 0xa2, 0x05, 0x51, 0xbf, 0x00, 0x00, 0x00, 0x00, 0x28,
  0xcd, 0x13, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
  ];
//unsigned int ___avion_model_dragger_arrow_mesh_len = 631;

pub static MESH_PLANE : &'static [u8] = &[
  0x04, 0x00, 0x6d, 0x65, 0x73, 0x68, 0x0d, 0x00, 0x64, 0x72, 0x61, 0x67,
  0x67, 0x65, 0x72, 0x5f, 0x70, 0x6c, 0x61, 0x6e, 0x65, 0x08, 0x00, 0x0c,
  0xd7, 0xa3, 0xbc, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0c,
  0xd7, 0xa3, 0xbc, 0x66, 0x66, 0xe6, 0x3e, 0x00, 0x00, 0x00, 0x00, 0x0c,
  0xd7, 0xa3, 0x3c, 0x66, 0x66, 0xe6, 0x3e, 0x00, 0x00, 0x00, 0x00, 0x0c,
  0xd7, 0xa3, 0x3c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0c,
  0xd7, 0xa3, 0xbc, 0x00, 0x00, 0x00, 0x00, 0x66, 0x66, 0xe6, 0x3e, 0x0c,
  0xd7, 0xa3, 0xbc, 0x66, 0x66, 0xe6, 0x3e, 0x66, 0x66, 0xe6, 0x3e, 0x0c,
  0xd7, 0xa3, 0x3c, 0x66, 0x66, 0xe6, 0x3e, 0x66, 0x66, 0xe6, 0x3e, 0x0c,
  0xd7, 0xa3, 0x3c, 0x00, 0x00, 0x00, 0x00, 0x66, 0x66, 0xe6, 0x3e, 0x0c,
  0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x01, 0x00, 0x04, 0x00, 0x05,
  0x00, 0x05, 0x00, 0x06, 0x00, 0x02, 0x00, 0x05, 0x00, 0x02, 0x00, 0x01,
  0x00, 0x06, 0x00, 0x07, 0x00, 0x03, 0x00, 0x06, 0x00, 0x03, 0x00, 0x02,
  0x00, 0x00, 0x00, 0x03, 0x00, 0x07, 0x00, 0x00, 0x00, 0x07, 0x00, 0x04,
  0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x03,
  0x00, 0x07, 0x00, 0x06, 0x00, 0x05, 0x00, 0x07, 0x00, 0x05, 0x00, 0x04,
  0x00, 0x08, 0x00, 0x28, 0xcd, 0x13, 0xbf, 0x28, 0xcd, 0x13, 0xbf, 0x28,
  0xcd, 0x13, 0xbf, 0x28, 0xcd, 0x13, 0xbf, 0x28, 0xcd, 0x13, 0x3f, 0x28,
  0xcd, 0x13, 0xbf, 0x28, 0xcd, 0x13, 0x3f, 0x28, 0xcd, 0x13, 0x3f, 0x28,
  0xcd, 0x13, 0xbf, 0x28, 0xcd, 0x13, 0x3f, 0x28, 0xcd, 0x13, 0xbf, 0x28,
  0xcd, 0x13, 0xbf, 0x28, 0xcd, 0x13, 0xbf, 0x28, 0xcd, 0x13, 0xbf, 0x28,
  0xcd, 0x13, 0x3f, 0x28, 0xcd, 0x13, 0xbf, 0x28, 0xcd, 0x13, 0x3f, 0x28,
  0xcd, 0x13, 0x3f, 0x28, 0xcd, 0x13, 0x3f, 0x28, 0xcd, 0x13, 0x3f, 0x28,
  0xcd, 0x13, 0x3f, 0x28, 0xcd, 0x13, 0x3f, 0x28, 0xcd, 0x13, 0xbf, 0x28,
  0xcd, 0x13, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00
];
//unsigned int ___avion_model_dragger_plane_mesh_len = 313;

