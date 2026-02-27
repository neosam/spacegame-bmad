use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use lyon_tessellation::geom::point;
use lyon_tessellation::path::Path;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
};

/// Generate the player ship mesh using lyon tessellation.
/// The ship is a recognizable spacecraft silhouette facing +Y.
/// `upgrade_tier` influences visual detail (1-5, tier 1 for Sprint 0).
pub fn generate_player_mesh(_upgrade_tier: u8) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    // Build ship path: a pointed triangle facing +Y with hull shape
    let mut builder = Path::builder();

    // Nose (top)
    builder.begin(point(0.0, 20.0));
    // Right wing
    builder.line_to(point(12.0, -14.0));
    // Right indent
    builder.line_to(point(5.0, -8.0));
    // Thruster right
    builder.line_to(point(5.0, -16.0));
    // Thruster left
    builder.line_to(point(-5.0, -16.0));
    // Left indent
    builder.line_to(point(-5.0, -8.0));
    // Left wing
    builder.line_to(point(-12.0, -14.0));
    builder.close();

    let path = builder.build();

    let result = tessellator.tessellate_path(
        &path,
        &FillOptions::default(),
        &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
            [vertex.position().x, vertex.position().y, 0.0]
        }),
    );

    if let Err(e) = result {
        warn!("Lyon tessellation failed: {e:?}, using fallback triangle");
        return generate_fallback_mesh();
    }

    let positions: Vec<[f32; 3]> = buffers.vertices.clone();
    let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| [(p[0] + 12.0) / 24.0, (p[1] + 16.0) / 36.0])
        .collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(buffers.indices));
    mesh
}

/// Generate a thin rectangle mesh for laser pulse rendering.
pub fn generate_laser_mesh(length: f32, width: f32) -> Mesh {
    Mesh::from(Rectangle::new(width, length))
}

/// Generate a small circle mesh for spread projectile rendering.
pub fn generate_projectile_mesh(radius: f32) -> Mesh {
    Mesh::from(Circle::new(radius))
}

/// Generate an irregular polygon mesh for asteroid rendering.
/// Uses lyon tessellation with random vertex offsets from a base circle.
pub fn generate_asteroid_mesh(radius: f32) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let mut builder = Path::builder();
    let vertex_count = 8;
    let angle_step = std::f32::consts::TAU / vertex_count as f32;

    // Generate irregular polygon vertices
    for i in 0..vertex_count {
        let angle = angle_step * i as f32;
        // Offset radius by a pseudo-random factor based on index for visual variety
        let offset = 1.0 - 0.2 * ((i * 7 + 3) % 5) as f32 / 4.0;
        let r = radius * offset;
        let x = angle.cos() * r;
        let y = angle.sin() * r;
        if i == 0 {
            builder.begin(point(x, y));
        } else {
            builder.line_to(point(x, y));
        }
    }
    builder.close();
    let path = builder.build();

    let result = tessellator.tessellate_path(
        &path,
        &FillOptions::default(),
        &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
            [vertex.position().x, vertex.position().y, 0.0]
        }),
    );

    if let Err(e) = result {
        warn!("Asteroid tessellation failed: {e:?}, using circle fallback");
        return Mesh::from(Circle::new(radius));
    }

    let positions: Vec<[f32; 3]> = buffers.vertices.clone();
    let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| [(p[0] / radius + 1.0) / 2.0, (p[1] / radius + 1.0) / 2.0])
        .collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(buffers.indices));
    mesh
}

/// Generate a diamond-shaped mesh for Scout Drone rendering.
pub fn generate_drone_mesh(radius: f32) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let mut builder = Path::builder();
    // Diamond shape: 4 points
    builder.begin(point(0.0, radius));           // Top
    builder.line_to(point(radius * 0.6, 0.0));   // Right
    builder.line_to(point(0.0, -radius));         // Bottom
    builder.line_to(point(-radius * 0.6, 0.0));  // Left
    builder.close();
    let path = builder.build();

    let result = tessellator.tessellate_path(
        &path,
        &FillOptions::default(),
        &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
            [vertex.position().x, vertex.position().y, 0.0]
        }),
    );

    if let Err(e) = result {
        warn!("Drone tessellation failed: {e:?}, using circle fallback");
        return Mesh::from(Circle::new(radius));
    }

    let positions: Vec<[f32; 3]> = buffers.vertices.clone();
    let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| [(p[0] / radius + 1.0) / 2.0, (p[1] / radius + 1.0) / 2.0])
        .collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(buffers.indices));
    mesh
}

/// Generate an asteroid mesh and verify it produces valid geometry.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_asteroid_mesh_produces_vertices() {
        let mesh = generate_asteroid_mesh(20.0);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("Asteroid mesh should have positions");
        let len = match positions {
            bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => panic!("Expected Float32x3 positions"),
        };
        assert!(len >= 3, "Asteroid mesh should have at least 3 vertices, got {len}");
        let indices = mesh.indices().expect("Asteroid mesh should have indices");
        let index_count = match indices {
            Indices::U32(v) => v.len(),
            _ => panic!("Expected U32 indices"),
        };
        assert!(index_count >= 3, "Asteroid mesh should have at least 3 indices, got {index_count}");
    }

    #[test]
    fn generate_drone_mesh_produces_vertices() {
        let mesh = generate_drone_mesh(10.0);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("Drone mesh should have positions");
        let len = match positions {
            bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => panic!("Expected Float32x3 positions"),
        };
        assert!(len >= 3, "Drone mesh should have at least 3 vertices, got {len}");
        let indices = mesh.indices().expect("Drone mesh should have indices");
        let index_count = match indices {
            Indices::U32(v) => v.len(),
            _ => panic!("Expected U32 indices"),
        };
        assert!(index_count >= 3, "Drone mesh should have at least 3 indices, got {index_count}");
    }
}

/// Fallback mesh if lyon tessellation fails (graceful degradation).
fn generate_fallback_mesh() -> Mesh {
    let positions = vec![
        [0.0, 20.0, 0.0],   // Nose
        [12.0, -14.0, 0.0], // Right
        [-12.0, -14.0, 0.0], // Left
    ];
    let normals = vec![[0.0, 0.0, 1.0]; 3];
    let uvs = vec![[0.5, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let indices = vec![0u32, 1, 2];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
