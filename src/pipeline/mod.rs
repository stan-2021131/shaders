pub mod clipping;
pub mod rasterizer;
pub mod shaders;

use crate::buffer::{DepthBuffer, Framebuffer, RenderBand, split_render_bands};
use crate::math::Mat4;
use crate::mesh::Mesh;
use clipping::{ClipTriangle, ClipVertex, ClippedTriangles, clip_triangle_near_plane};
pub use rasterizer::{BlendMode, VertexOut, is_front_facing, rasterize_triangle, rasterize_triangle_band};
pub use shaders::*;

/// Contexto de transformación y sombreado para el renderizado de una malla.
pub struct RenderContext<'a, S: FragmentShader> {
    pub model_matrix: &'a Mat4,
    pub rotation_matrix: &'a Mat4,
    pub view_proj_matrix: &'a Mat4,
    pub shader: &'a S,
    pub blend_mode: BlendMode,
}

impl<'a, S: FragmentShader> RenderContext<'a, S> {
    pub fn new(
        model_matrix: &'a Mat4,
        rotation_matrix: &'a Mat4,
        view_proj_matrix: &'a Mat4,
        shader: &'a S,
    ) -> Self {
        Self {
            model_matrix,
            rotation_matrix,
            view_proj_matrix,
            shader,
            blend_mode: BlendMode::Opaque,
        }
    }

    pub fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }
}

pub struct RenderTarget<'a> {
    pub framebuffer: &'a mut Framebuffer,
    pub depth_buffer: &'a mut DepthBuffer,
}

pub struct SoftwareRenderer {
    vertex_clip_cache: Vec<ClipVertex>,
    triangles_cache: Vec<(VertexOut, VertexOut, VertexOut)>,
}

impl SoftwareRenderer {
    pub fn new() -> Self {
        Self {
            vertex_clip_cache: Vec::with_capacity(1024),
            triangles_cache: Vec::with_capacity(1024),
        }
    }

    /// Renderiza una malla indexada ejecutando el pipeline completo por software en CPU:
    /// 1. Transformación de modelo y rotación de normales.
    /// 2. Transformación de vista y proyección homogénea.
    /// 3. Clipping contra el plano cercano (near plane).
    /// 4. División de perspectiva y mapeo a pantalla (viewport).
    /// 5. Backface culling en espacio de pantalla.
    /// 6. Rasterización paralela con prueba de profundidad y shader de fragmentos.
    pub fn render_mesh<S: FragmentShader>(
        &mut self,
        mesh: &Mesh,
        ctx: &RenderContext<'_, S>,
        target: &mut RenderTarget<'_>,
    ) {
        self.vertex_clip_cache.clear();
        if self.vertex_clip_cache.capacity() < mesh.vertices.len() {
            self.vertex_clip_cache.reserve(mesh.vertices.len() - self.vertex_clip_cache.capacity());
        }

        // Transformación de vértices
        for v in &mesh.vertices {
            let local_pos = v.position;
            let world_pos = ctx.model_matrix.transform_point(local_pos);
            let world_normal = ctx.rotation_matrix.transform_direction(v.normal).normalize();
            let world_tangent = ctx.rotation_matrix.transform_direction(v.tangent).normalize();
            let clip_pos = ctx.view_proj_matrix.transform_vec4(world_pos.to_vec4(1.0));

            self.vertex_clip_cache.push(ClipVertex::new(
                clip_pos,
                world_pos,
                world_normal,
                world_tangent,
                v.uv,
            ));
        }

        let width = target.framebuffer.width as f32;
        let height = target.framebuffer.height as f32;

        self.triangles_cache.clear();

        // Clipping contra el near plane y ensamblado de primitivas
        for chunk in mesh.indices.chunks_exact(3) {
            let cv0 = self.vertex_clip_cache[chunk[0] as usize];
            let cv1 = self.vertex_clip_cache[chunk[1] as usize];
            let cv2 = self.vertex_clip_cache[chunk[2] as usize];

            let tri = ClipTriangle::new(cv0, cv1, cv2);

            match clip_triangle_near_plane(&tri) {
                ClippedTriangles::Zero => {}
                ClippedTriangles::One(t) => {
                    self.collect_visible_triangle(&t, width, height);
                }
                ClippedTriangles::Two(t1, t2) => {
                    self.collect_visible_triangle(&t1, width, height);
                    self.collect_visible_triangle(&t2, width, height);
                }
            }
        }

        if self.triangles_cache.is_empty() || target.framebuffer.height == 0 {
            return;
        }

        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .clamp(1, target.framebuffer.height);

        if num_threads == 1 || self.triangles_cache.len() <= 2 {
            for (vo0, vo1, vo2) in &self.triangles_cache {
                let fb_width = target.framebuffer.width;
                let fb_height = target.framebuffer.height;
                let mut band = RenderBand {
                    width: fb_width,
                    height: fb_height,
                    y_start: 0,
                    y_end: fb_height,
                    pixels: target.framebuffer.as_mut_slice(),
                    depth: target.depth_buffer.as_mut_slice(),
                };
                rasterize_triangle_band(vo0, vo1, vo2, &mut band, ctx.shader, ctx.blend_mode);
            }
        } else {
            let bands = split_render_bands(
                target.framebuffer,
                target.depth_buffer,
                num_threads,
            );

            let triangles = &self.triangles_cache;
            let shader = ctx.shader;
            let blend_mode = ctx.blend_mode;

            std::thread::scope(|s| {
                for mut band in bands {
                    s.spawn(move || {
                        for (vo0, vo1, vo2) in triangles {
                            rasterize_triangle_band(
                                vo0,
                                vo1,
                                vo2,
                                &mut band,
                                shader,
                                blend_mode,
                            );
                        }
                    });
                }
            });
        }
    }

    #[inline]
    fn collect_visible_triangle(&mut self, tri: &ClipTriangle, width: f32, height: f32) {
        let vo0 = VertexOut::from_clip_vertex(&tri.v[0], width, height);
        let vo1 = VertexOut::from_clip_vertex(&tri.v[1], width, height);
        let vo2 = VertexOut::from_clip_vertex(&tri.v[2], width, height);

        if is_front_facing(&vo0, &vo1, &vo2) {
            self.triangles_cache.push((vo0, vo1, vo2));
        }
    }
}

