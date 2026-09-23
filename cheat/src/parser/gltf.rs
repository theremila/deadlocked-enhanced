use std::{fs, path::Path};

use glam::Vec3;
use serde::Deserialize;

use super::bvh::{Bvh, Surface, Triangle};

#[derive(Deserialize)]
struct Gltf {
    accessors: Vec<Accessor>,
    #[serde(rename = "bufferViews")]
    buffer_views: Vec<BufferView>,
    buffers: Vec<Buffer>,
    meshes: Vec<Mesh>,
    nodes: Vec<Node>,
}

#[derive(Deserialize)]
struct Accessor {
    #[serde(rename = "bufferView")]
    buffer_view: usize,
    #[serde(rename = "byteOffset", default)]
    byte_offset: usize,
    #[serde(rename = "componentType")]
    component_type: u32,
    count: usize,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct BufferView {
    buffer: usize,
    #[serde(rename = "byteOffset", default)]
    byte_offset: usize,
    #[serde(rename = "byteStride")]
    byte_stride: Option<usize>,
}

#[derive(Deserialize)]
struct Buffer {
    uri: String,
}

#[derive(Deserialize)]
struct Mesh {
    primitives: Vec<Primitive>,
}

#[derive(Deserialize)]
struct Primitive {
    attributes: Attributes,
    indices: usize,
}

#[derive(Deserialize)]
struct Attributes {
    #[serde(rename = "POSITION")]
    position: usize,
}

#[derive(Deserialize)]
struct Node {
    mesh: Option<usize>,
    extras: Option<NodeExtras>,
}

#[derive(Deserialize)]
struct NodeExtras {
    #[serde(rename = "SurfaceProperty", default)]
    surface_property: String,
    #[serde(rename = "InteractAs", default)]
    interact_as: Vec<String>,
}

pub fn load_material_bvh(path: &Path) -> Option<Bvh> {
    let document: Gltf = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    let parent = path.parent()?;
    let buffers: Vec<Vec<u8>> = document
        .buffers
        .iter()
        .map(|buffer| fs::read(parent.join(&buffer.uri)).ok())
        .collect::<Option<_>>()?;
    let mut triangles = Vec::new();

    for node in &document.nodes {
        let (Some(mesh_index), Some(extras)) = (node.mesh, &node.extras) else {
            continue;
        };
        // Clip-only shapes do not stop bullets. passbullets is deliberately omitted too.
        if !extras.interact_as.is_empty() {
            continue;
        }
        let surface = Surface::from_name(&extras.surface_property);
        let mesh = document.meshes.get(mesh_index)?;

        for primitive in &mesh.primitives {
            let vertices = read_positions(&document, &buffers, primitive.attributes.position)?;
            let indices = read_indices(&document, &buffers, primitive.indices)?;
            for face in indices.chunks_exact(3) {
                let Some((&v0, &v1, &v2)) = vertices
                    .get(face[0] as usize)
                    .zip(vertices.get(face[1] as usize))
                    .zip(vertices.get(face[2] as usize))
                    .map(|((v0, v1), v2)| (v0, v1, v2))
                else {
                    continue;
                };
                triangles.push(Triangle {
                    v0,
                    v1,
                    v2,
                    surface,
                });
            }
        }
    }

    if triangles.is_empty() {
        return None;
    }
    let mut bvh = Bvh::new();
    bvh.set(triangles);
    bvh.build();
    Some(bvh)
}

fn accessor_bytes<'document, 'buffer>(
    document: &'document Gltf,
    buffers: &'buffer [Vec<u8>],
    accessor_index: usize,
) -> Option<(&'buffer [u8], &'document Accessor, usize)> {
    let accessor = document.accessors.get(accessor_index)?;
    let view = document.buffer_views.get(accessor.buffer_view)?;
    let buffer = buffers.get(view.buffer)?;
    let offset = view.byte_offset.checked_add(accessor.byte_offset)?;
    Some((
        buffer.get(offset..)?,
        accessor,
        view.byte_stride.unwrap_or(0),
    ))
}

fn read_positions(document: &Gltf, buffers: &[Vec<u8>], index: usize) -> Option<Vec<Vec3>> {
    let (bytes, accessor, stride) = accessor_bytes(document, buffers, index)?;
    if accessor.component_type != 5126 || accessor.kind != "VEC3" {
        return None;
    }
    let stride = if stride == 0 { 12 } else { stride };
    (0..accessor.count)
        .map(|index: usize| {
            let start = index.checked_mul(stride)?;
            let bytes = bytes.get(start..start + 12)?;
            Some(Vec3::new(
                f32::from_le_bytes(bytes[0..4].try_into().ok()?),
                f32::from_le_bytes(bytes[4..8].try_into().ok()?),
                f32::from_le_bytes(bytes[8..12].try_into().ok()?),
            ))
        })
        .collect()
}

fn read_indices(document: &Gltf, buffers: &[Vec<u8>], index: usize) -> Option<Vec<u32>> {
    let (bytes, accessor, stride) = accessor_bytes(document, buffers, index)?;
    if accessor.kind != "SCALAR" {
        return None;
    }
    let component_size = match accessor.component_type {
        5121 => 1,
        5123 => 2,
        5125 => 4,
        _ => return None,
    };
    let stride = if stride == 0 { component_size } else { stride };
    (0..accessor.count)
        .map(|index: usize| {
            let start = index.checked_mul(stride)?;
            let bytes = bytes.get(start..start + component_size)?;
            Some(match component_size {
                1 => bytes[0] as u32,
                2 => u16::from_le_bytes(bytes.try_into().ok()?) as u32,
                4 => u32::from_le_bytes(bytes.try_into().ok()?),
                _ => unreachable!(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_source_surface_names() {
        assert_eq!(Surface::from_name("wood_plank"), Surface::Wood);
        assert_eq!(Surface::from_name("metalpanel"), Surface::Metal);
        assert_eq!(Surface::from_name("chainlink"), Surface::Grate);
    }

    #[test]
    fn loads_surface_from_gltf_extras() {
        let directory =
            std::env::temp_dir().join(format!("deadlocked-gltf-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let mut binary = Vec::new();
        for value in [10.0_f32, -10.0, -10.0, 10.0, 10.0, -10.0, 10.0, 0.0, 10.0] {
            binary.extend_from_slice(&value.to_le_bytes());
        }
        for index in [0_u32, 1, 2] {
            binary.extend_from_slice(&index.to_le_bytes());
        }
        std::fs::write(directory.join("mesh.bin"), binary).unwrap();
        std::fs::write(
            directory.join("mesh.gltf"),
            r#"{
                "accessors":[
                    {"bufferView":0,"componentType":5126,"count":3,"type":"VEC3"},
                    {"bufferView":1,"componentType":5125,"count":3,"type":"SCALAR"}
                ],
                "bufferViews":[
                    {"buffer":0,"byteLength":36,"byteOffset":0},
                    {"buffer":0,"byteLength":12,"byteOffset":36}
                ],
                "buffers":[{"byteLength":48,"uri":"mesh.bin"}],
                "meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1}]}],
                "nodes":[{"mesh":0,"extras":{"SurfaceProperty":"wood_plank","InteractAs":[]}}]
            }"#,
        )
        .unwrap();

        let bvh = load_material_bvh(&directory.join("mesh.gltf")).unwrap();
        assert_eq!(
            bvh.segment_intersections(Vec3::ZERO, Vec3::new(20.0, 0.0, 0.0)),
            vec![(10.0, Surface::Wood, false)]
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    #[ignore = "requires MATERIAL_GLTF_PATH exported by Source2Viewer"]
    fn loads_exported_material_map() {
        let path = std::env::var_os("MATERIAL_GLTF_PATH").unwrap();
        let bvh = load_material_bvh(Path::new(&path)).unwrap();
        let cache = std::env::temp_dir().join(format!(
            "deadlocked-material-cache-{}.bvh",
            uuid::Uuid::new_v4()
        ));
        bvh.save(&cache).unwrap();
        assert!(Bvh::load(&cache).is_some());
        std::fs::remove_file(cache).unwrap();
    }
}
