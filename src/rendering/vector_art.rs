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
