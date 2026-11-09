use crate::math::{Vec2, Vec3};
use std::path::Path;

/// Vértice individual con atributos geométricos esenciales para el pipeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub tangent: Vec3,
}

impl Vertex {
    pub const fn new(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self {
            position,
            normal,
            uv,
            tangent: Vec3::new(1.0, 0.0, 0.0),
        }
    }

    pub const fn with_tangent(position: Vec3, normal: Vec3, uv: Vec2, tangent: Vec3) -> Self {
        Self {
            position,
            normal,
            uv,
            tangent,
        }
    }
}

/// Malla poligonal 3D indexada cargada en memoria.
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    /// Carga una malla tridimensional desde un archivo Wavefront OBJ en disco.
    /// Extrae posiciones, normales y coordenadas UV ignorando materiales y texturas
    /// conforme a los requisitos de shaders puros.
    pub fn load_obj<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_ref = path.as_ref();
        let load_options = tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        };

        let (models, _) = tobj::load_obj(path_ref, &load_options)
            .map_err(|e| format!("Error al cargar modelo OBJ {:?}: {}", path_ref, e))?;

        if models.is_empty() {
            return Err(format!("El archivo OBJ {:?} no contiene modelos", path_ref));
        }

        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();

        for model in &models {
            let mesh = &model.mesh;
            let index_offset = all_vertices.len() as u32;

            let num_vertices = mesh.positions.len() / 3;
            let has_normals = !mesh.normals.is_empty();
            let has_uvs = !mesh.texcoords.is_empty();

            for i in 0..num_vertices {
                let pos = Vec3::new(
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                );

                let norm = if has_normals && (i * 3 + 2) < mesh.normals.len() {
                    Vec3::new(
                        mesh.normals[i * 3],
                        mesh.normals[i * 3 + 1],
                        mesh.normals[i * 3 + 2],
                    )
                    .normalize()
                } else {
                    Vec3::up()
                };

                let uv = if has_uvs && (i * 2 + 1) < mesh.texcoords.len() {
                    Vec2::new(mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1])
                } else {
                    Vec2::zero()
                };

                all_vertices.push(Vertex::new(pos, norm, uv));
            }

            for &idx in &mesh.indices {
                all_indices.push(idx + index_offset);
            }
        }

        if all_indices.len() < 3 || all_vertices.is_empty() {
            return Err(format!(
                "El archivo OBJ {:?} no contiene geometría triangular utilizable",
                path_ref
            ));
        }

        Ok(Self::new(all_vertices, all_indices))
    }
}

