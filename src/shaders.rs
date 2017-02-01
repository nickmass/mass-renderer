use ::{
    V3,
    V4,
    M3,
    M4,
    v3,
    Matrix,
    SquareMatrix,
    InnerSpace,
};

use ::renderer::{Shader, RenderContext, Surface, matrix_transform};
use ::model::Face;

pub struct SolidShader {
    light_dir: V3,
    intensity: V3,
    transform: M4,
}

impl SolidShader {
    pub fn new(light_dir: V3) -> SolidShader {
        SolidShader {
            light_dir: light_dir.normalize(),
            intensity: V3::unit_z(),
            transform: M4::identity(),
        }
    }
}

impl Shader for SolidShader {
    fn prepare(&mut self, ctx: &RenderContext) {
        self.transform = ctx.viewport * ctx.projection * ctx.modelview;
    }

    fn vertex(&mut self, _ctx: &RenderContext, face: &Face, vert: usize) -> V4 {
        self.intensity[vert] = face.norms[vert].dot(self.light_dir);
        self.transform * face.verts[vert].extend(1.)

    }

    fn fragment(&mut self, _ctx: &RenderContext, coords: V3) -> Option<V3> {
        let intensity = self.intensity.dot(coords).max(0.0);
        let c = v3(1.,1.,1.) * intensity;
        Some(c)
    }
}

pub struct DefaultShader {
    light_dir: V3,
    light_depth: Surface<f64>,
    light_matrix: M4,
    transform: M4,
    pm: M4,
    pm_t: M4,
    uv: M3,
    norm: M3,
    ndc_coords: M3,
    shadow_coords: M3,
}

impl DefaultShader {
    pub fn new(light_dir: V3, light_depth: Surface<f64>, light_matrix: M4) -> DefaultShader {
        DefaultShader {
            light_dir: light_dir.normalize(),
            light_depth: light_depth,
            light_matrix: light_matrix,
            transform: M4::identity(),
            pm: M4::identity(),
            pm_t: M4::identity(),
            uv: M3::identity(),
            norm: M3::identity(),
            ndc_coords: M3::identity(),
            shadow_coords: M3::identity(),
        }
    }
}

impl Shader for DefaultShader {
    fn prepare(&mut self, ctx: &RenderContext) {
        self.transform = ctx.viewport * ctx.projection * ctx.modelview;
        self.pm = ctx.projection * ctx.modelview;
        self.pm_t = self.pm.transpose().invert().unwrap();
    }

    fn vertex(&mut self, _ctx: &RenderContext, face: &Face, vert: usize) -> V4 {
        self.uv[vert] = face.texs[vert].extend(1.);
        self.norm[vert] = (self.pm_t * face.norms[vert].extend(0.)).truncate();
        self.shadow_coords[vert] = matrix_transform(face.verts[vert], self.light_matrix);

        let next_vert = self.transform * face.verts[vert].extend(1.);
        self.ndc_coords[vert] = (next_vert / next_vert.w).truncate();

        next_vert
    }

    fn fragment(&mut self, ctx: &RenderContext, coords: V3) -> Option<V3> {
        let norm = (self.norm * coords).normalize();
        let uv = (self.uv * coords).truncate();

        let shadow_c = self.shadow_coords * coords;

        let shadow = if self.light_depth.is_in_bounds(shadow_c.x as u32, shadow_c.y as u32)
            && shadow_c.x >= 0. && shadow_c.y >= 0. {
                if self.light_depth.get(shadow_c.x as u32,
                                        shadow_c.y as u32) < shadow_c.z + 0.02 {
                    1.0
                } else {
                    0.3

                }
            } else {
                0.3
            };

        let a = M3::from_cols(
            self.ndc_coords[1] - self.ndc_coords[0],
            self.ndc_coords[2] - self.ndc_coords[0],
            norm,
        ).transpose();

        let ai = a.invert().unwrap();
        let i = ai * v3(self.uv[1].x - self.uv[0].x, self.uv[2].x - self.uv[0].x, 0.);
        let j = ai * v3(self.uv[1].y - self.uv[0].y, self.uv[2].y - self.uv[0].y, 0.);

        let b = M3::from_cols(
            i.normalize(),
            j.normalize(),
            norm,
        );

        let n = (b * ctx.model.normal(uv)).normalize();

        let l = matrix_transform(self.light_dir, self.pm).normalize();
        let r = ((n * n.dot(l * 2.)) - l).normalize();
        let diffuse = n.dot(l).max(0.0);
        let specular = r.z.max(0.0).powf(ctx.model.specular(uv));
        let c = ctx.model.diffuse(uv);
        if c.w <= 0.0 { return None; }
        let mut c = c.truncate() * (diffuse + 0.6 * specular) * shadow;
        for i in 0..3 {
            c[i] = (c[i] + 0.02).min(1.);
        }

        Some(c)
    }
}

pub struct DepthShader {
    transform: M4,
}

impl DepthShader {
    pub fn new() -> DepthShader {
        DepthShader {
            transform: M4::identity(),
        }
    }
}

impl Shader for DepthShader {
    fn prepare(&mut self, ctx: &RenderContext) {
        self.transform = ctx.viewport * ctx.projection * ctx.modelview;
    }

    fn vertex(&mut self, _ctx: &RenderContext, face: &Face, vert: usize) -> V4 {
        self.transform * face.verts[vert].extend(1.)
    }

    fn fragment(&mut self, _ctx: &RenderContext, _coords: V3) -> Option<V3> {
        Some(v3(0.,0.,0.,))
    }
}
