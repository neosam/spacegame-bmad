use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use lyon_tessellation::geom::point;
use lyon_tessellation::path::Path;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
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

/// Generate a hexagon mesh for TutorialStation rendering (teal color applied separately).
pub fn generate_tutorial_station_mesh(radius: f32) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let mut builder = Path::builder();
    let vertex_count = 6u32;
    let angle_step = std::f32::consts::TAU / vertex_count as f32;

    for i in 0..vertex_count {
        // Flat-top hexagon: start at angle 0 (right side)
        let angle = angle_step * i as f32;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
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
        warn!("TutorialStation tessellation failed: {e:?}, using circle fallback");
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

/// Generate an irregular polygon mesh for TutorialWreck rendering (dark-grey color applied separately).
pub fn generate_tutorial_wreck_mesh(radius: f32) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let mut builder = Path::builder();
    // Irregular 7-point polygon — uneven offsets give wreck/damage feel
    let vertex_count = 7usize;
    let angle_step = std::f32::consts::TAU / vertex_count as f32;
    // Pseudo-random radii to look damaged/irregular
    let offsets: [f32; 7] = [0.85, 0.65, 1.0, 0.55, 0.90, 0.70, 0.80];

    for i in 0..vertex_count {
        let angle = angle_step * i as f32;
        let r = radius * offsets[i];
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
        warn!("TutorialWreck tessellation failed: {e:?}, using circle fallback");
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

/// Generate a diamond-shaped mesh for GravityWellGenerator rendering (orange color applied separately).
/// Wider than the Scout Drone diamond to be visually distinct.
pub fn generate_tutorial_generator_mesh(radius: f32) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let mut builder = Path::builder();
    // Wider diamond: 4 points with equal horizontal/vertical extent
    builder.begin(point(0.0, radius));           // Top
    builder.line_to(point(radius, 0.0));         // Right (wider than drone)
    builder.line_to(point(0.0, -radius));        // Bottom
    builder.line_to(point(-radius, 0.0));        // Left
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
        warn!("GravityWellGenerator tessellation failed: {e:?}, using circle fallback");
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

/// Generate a small diamond mesh for material drop pickups.
/// Uses the same pattern as generate_tutorial_generator_mesh but with a smaller radius.
pub fn generate_material_drop_mesh(half_size: f32) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let mut builder = Path::builder();
    builder.begin(point(0.0, half_size));          // Top
    builder.line_to(point(half_size, 0.0));        // Right
    builder.line_to(point(0.0, -half_size));       // Bottom
    builder.line_to(point(-half_size, 0.0));       // Left
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
        warn!("Material drop tessellation failed: {e:?}, using circle fallback");
        return Mesh::from(Circle::new(half_size));
    }

    let positions: Vec<[f32; 3]> = buffers.vertices.clone();
    let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| [(p[0] / half_size + 1.0) / 2.0, (p[1] / half_size + 1.0) / 2.0])
        .collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(buffers.indices));
    mesh
}

/// Generate an arrowhead mesh for Fighter rendering.
/// Wider and more aggressive than the Scout Drone diamond, facing +Y.
pub fn generate_fighter_mesh(radius: f32) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let mut builder = Path::builder();
    // Bold arrowhead: wide base, sharp nose
    let h = radius;
    let w = radius * 0.9;
    builder.begin(point(0.0, h));           // Nose (top)
    builder.line_to(point(w, -h * 0.5));    // Right base
    builder.line_to(point(w * 0.3, -h * 0.1)); // Right notch
    builder.line_to(point(-w * 0.3, -h * 0.1)); // Left notch
    builder.line_to(point(-w, -h * 0.5));   // Left base
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
        warn!("Fighter tessellation failed: {e:?}, using circle fallback");
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

/// Generate a large octagon mesh for Heavy Cruiser rendering.
/// Large and imposing — clearly the biggest enemy type.
pub fn generate_heavy_cruiser_mesh(radius: f32) -> Mesh {
    let vertex_count = 8u32;
    let angle_step = std::f32::consts::TAU / vertex_count as f32;
    // Rotate 22.5° so flat sides face up/down (more imposing silhouette)
    let offset = std::f32::consts::PI / 8.0;

    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    let mut builder = Path::builder();

    for i in 0..vertex_count {
        let angle = angle_step * i as f32 + offset;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
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
        warn!("HeavyCruiser tessellation failed: {e:?}, using circle fallback");
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

/// Generate an elongated needle mesh for Sniper rendering.
/// Thin and precise — visually communicates long-range accuracy.
pub fn generate_sniper_mesh(radius: f32) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let mut builder = Path::builder();
    // Long thin diamond: 4:1 aspect ratio, facing +Y
    let half_w = radius * 0.25;
    let half_h = radius;
    builder.begin(point(0.0, half_h));     // Top needle tip
    builder.line_to(point(half_w, 0.0));   // Right mid
    builder.line_to(point(0.0, -half_h));  // Bottom tail
    builder.line_to(point(-half_w, 0.0)); // Left mid
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
        warn!("Sniper tessellation failed: {e:?}, using circle fallback");
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

/// Generate a rectangular mesh with small wing stubs for Trader Ship rendering.
/// Neutral, boxy silhouette — clearly not a combat vessel.
pub fn generate_trader_mesh(radius: f32) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let mut builder = Path::builder();
    // Box hull with small side wings
    let hw = radius * 0.5;   // half-width hull
    let hh = radius * 0.7;   // half-height hull
    let ww = radius * 0.85;  // wing tip x
    let wy = radius * 0.0;   // wing tip y

    builder.begin(point(0.0, hh));      // Nose tip
    builder.line_to(point(hw, hh * 0.3));  // Right shoulder
    builder.line_to(point(ww, wy));         // Right wing tip
    builder.line_to(point(hw, -hh * 0.3)); // Right hip
    builder.line_to(point(hw * 0.5, -hh)); // Right tail
    builder.line_to(point(-hw * 0.5, -hh));// Left tail
    builder.line_to(point(-hw, -hh * 0.3));// Left hip
    builder.line_to(point(-ww, wy));        // Left wing tip
    builder.line_to(point(-hw, hh * 0.3)); // Left shoulder
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
        warn!("TraderShip tessellation failed: {e:?}, using circle fallback");
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

    #[test]
    fn generate_tutorial_station_mesh_produces_vertices() {
        let mesh = generate_tutorial_station_mesh(20.0);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("TutorialStation mesh should have positions");
        let len = match positions {
            bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => panic!("Expected Float32x3 positions"),
        };
        assert!(len >= 3, "TutorialStation mesh should have at least 3 vertices, got {len}");
        let indices = mesh.indices().expect("TutorialStation mesh should have indices");
        let index_count = match indices {
            Indices::U32(v) => v.len(),
            _ => panic!("Expected U32 indices"),
        };
        assert!(index_count >= 3, "TutorialStation mesh should have at least 3 indices, got {index_count}");
    }

    #[test]
    fn generate_tutorial_wreck_mesh_produces_vertices() {
        let mesh = generate_tutorial_wreck_mesh(18.0);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("TutorialWreck mesh should have positions");
        let len = match positions {
            bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => panic!("Expected Float32x3 positions"),
        };
        assert!(len >= 3, "TutorialWreck mesh should have at least 3 vertices, got {len}");
        let indices = mesh.indices().expect("TutorialWreck mesh should have indices");
        let index_count = match indices {
            Indices::U32(v) => v.len(),
            _ => panic!("Expected U32 indices"),
        };
        assert!(index_count >= 3, "TutorialWreck mesh should have at least 3 indices, got {index_count}");
    }

    #[test]
    fn generate_fighter_mesh_produces_vertices() {
        let mesh = generate_fighter_mesh(12.0);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("Fighter mesh should have positions");
        let len = match positions {
            bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => panic!("Expected Float32x3 positions"),
        };
        assert!(len >= 3, "Fighter mesh should have at least 3 vertices, got {len}");
    }

    #[test]
    fn generate_heavy_cruiser_mesh_produces_vertices() {
        let mesh = generate_heavy_cruiser_mesh(22.0);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("HeavyCruiser mesh should have positions");
        let len = match positions {
            bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => panic!("Expected Float32x3 positions"),
        };
        assert!(len >= 3, "HeavyCruiser mesh should have at least 3 vertices, got {len}");
    }

    #[test]
    fn generate_sniper_mesh_produces_vertices() {
        let mesh = generate_sniper_mesh(10.0);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("Sniper mesh should have positions");
        let len = match positions {
            bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => panic!("Expected Float32x3 positions"),
        };
        assert!(len >= 3, "Sniper mesh should have at least 3 vertices, got {len}");
    }

    #[test]
    fn generate_trader_mesh_produces_vertices() {
        let mesh = generate_trader_mesh(14.0);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("TraderShip mesh should have positions");
        let len = match positions {
            bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => panic!("Expected Float32x3 positions"),
        };
        assert!(len >= 3, "TraderShip mesh should have at least 3 vertices, got {len}");
    }

    #[test]
    fn generate_tutorial_generator_mesh_produces_vertices() {
        let mesh = generate_tutorial_generator_mesh(25.0);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("GravityWellGenerator mesh should have positions");
        let len = match positions {
            bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => panic!("Expected Float32x3 positions"),
        };
        assert!(len >= 3, "GravityWellGenerator mesh should have at least 3 vertices, got {len}");
        let indices = mesh.indices().expect("GravityWellGenerator mesh should have indices");
        let index_count = match indices {
            Indices::U32(v) => v.len(),
            _ => panic!("Expected U32 indices"),
        };
        assert!(index_count >= 3, "GravityWellGenerator mesh should have at least 3 indices, got {index_count}");
    }
}

/// Generate a circle outline (ring) mesh using lyon stroke tessellation.
/// The resulting mesh is a thin ring centered at the origin with the given `radius`.
/// `stroke_width` controls the thickness of the ring.
/// This is used for the gravity well boundary visual indicator.
pub fn generate_circle_outline_mesh(radius: f32, stroke_width: f32) -> Mesh {
    let mut geometry: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut stroke_tess = StrokeTessellator::new();

    // Approximate a circle with 64 line segments for smooth appearance.
    let segment_count = 64usize;
    let angle_step = std::f32::consts::TAU / segment_count as f32;

    let mut builder = Path::builder();
    for i in 0..segment_count {
        let angle = angle_step * i as f32;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        if i == 0 {
            builder.begin(point(x, y));
        } else {
            builder.line_to(point(x, y));
        }
    }
    builder.close();
    let path = builder.build();

    let result = stroke_tess.tessellate_path(
        &path,
        &StrokeOptions::default().with_line_width(stroke_width),
        &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
            [vertex.position().x, vertex.position().y, 0.0]
        }),
    );

    if let Err(e) = result {
        warn!("Circle outline stroke tessellation failed: {e:?}, using circle fallback");
        return Mesh::from(Circle::new(radius));
    }

    let positions: Vec<[f32; 3]> = geometry.vertices.clone();
    let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| [(p[0] / radius + 1.0) / 2.0, (p[1] / radius + 1.0) / 2.0])
        .collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(geometry.indices));
    mesh
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
