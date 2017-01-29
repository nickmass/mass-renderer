extern crate image;
extern crate cgmath;

use cgmath::{
    InnerSpace,
    SquareMatrix,
    vec2 as v2,
    vec3 as v3,
    vec4 as v4,
    Vector2,
    Vector3,
    Vector4,
    Matrix4,
};

type V2 = Vector2<f64>;
type V3 = Vector3<f64>;
type V4 = Vector4<f64>;
type M4 = Matrix4<f64>;

pub mod renderer;
use renderer::Renderer;

pub mod model;
use model::Model;

fn main() {
    let models = head();

    let (width, height) = (1024, 1024);

    let eye = v3(1., 0., 3.);
    let center = v3(0., 0., 0.);
    let up = v3(0., 1., 0.);

    let mut renderer = Renderer::new(width, height);
    renderer.viewport(width as f64 / 8.,
                      height as f64 / 8.,
                      width as f64 * 0.75,
                      height as f64 * 0.75);
    renderer.projection((eye-center).magnitude());
    renderer.lookat(eye, center, up);
    renderer.light_direction(v3(1.,1.,1.));

    renderer.clear(v3(0.8,0.8,1.));
    for model in models.iter().chain(floor().iter())  {
        renderer.render(&model);
    }

    renderer.dump();
}

#[allow(dead_code)]
fn floor() -> Vec<Model> {
    vec![
        Model::load("../../tinyrenderer/obj/floor.obj",
                   "../../tinyrenderer/obj/floor_diffuse.tga",
                   "../../tinyrenderer/obj/floor_diffuse.tga",
                   "../../tinyrenderer/obj/floor_nm_tangent.tga")
    ]
}

#[allow(dead_code)]
fn head() -> Vec<Model> {
    vec![
        //Model::load("../../tinyrenderer/obj/african_head/african_head_eye_outer.obj",
        //           "../../tinyrenderer/obj/african_head/african_head_eye_outer_diffuse.tga",
        //           "../../tinyrenderer/obj/african_head/african_head_eye_outer_spec.tga",
        //           "../../tinyrenderer/obj/african_head/african_head_eye_outer_nm_tangent.tga"),
        Model::load("../../tinyrenderer/obj/african_head/african_head_eye_inner.obj",
                   "../../tinyrenderer/obj/african_head/african_head_eye_inner_diffuse.tga",
                   "../../tinyrenderer/obj/african_head/african_head_eye_inner_spec.tga",
                   "../../tinyrenderer/obj/african_head/african_head_eye_inner_nm_tangent.tga"),
        Model::load("../../tinyrenderer/obj/african_head/african_head.obj",
                   "../../tinyrenderer/obj/african_head/african_head_diffuse.tga",
                   "../../tinyrenderer/obj/african_head/african_head_spec.tga",
                   "../../tinyrenderer/obj/african_head/african_head_nm_tangent.tga"),
    ]
}
#[allow(dead_code)]
fn diablo() -> Vec<Model> {
    vec![
        Model::load("../../tinyrenderer/obj/diablo3_pose/diablo3_pose.obj",
                   "../../tinyrenderer/obj/diablo3_pose/diablo3_pose_diffuse.tga",
                   "../../tinyrenderer/obj/diablo3_pose/diablo3_pose_spec.tga",
                   "../../tinyrenderer/obj/diablo3_pose/diablo3_pose_nm_tangent.tga"),
    ]
}

#[allow(dead_code)]
fn boggie() -> Vec<Model> {
    vec![
        Model::load("../../tinyrenderer/obj/boggie/body.obj",
                   "../../tinyrenderer/obj/boggie/body_diffuse.tga",
                   "../../tinyrenderer/obj/boggie/body_spec.tga",
                   "../../tinyrenderer/obj/boggie/body_nm_tangent.tga"),
        Model::load("../../tinyrenderer/obj/boggie/eyes.obj",
                   "../../tinyrenderer/obj/boggie/eyes_diffuse.tga",
                   "../../tinyrenderer/obj/boggie/eyes_spec.tga",
                   "../../tinyrenderer/obj/boggie/eyes_nm_tangent.tga"),
        Model::load("../../tinyrenderer/obj/boggie/head.obj",
                   "../../tinyrenderer/obj/boggie/head_diffuse.tga",
                   "../../tinyrenderer/obj/boggie/head_spec.tga",
                   "../../tinyrenderer/obj/boggie/head_nm_tangent.tga"),
    ]
}
