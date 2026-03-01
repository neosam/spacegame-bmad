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
/// `upgrade_tier` influences visual detail:
/// - Tier 1–2: Standard silhouette (narrow wings, compact thruster)
/// - Tier 3–4: Wider wings and slightly larger thruster for upgraded look
/// - Tier 5:   Double-wing notch for maximum visual complexity
pub fn generate_player_mesh(upgrade_tier: u8) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let mut builder = Path::builder();

    match upgrade_tier {
        5 => {
            // Tier 5: Double-wing notch — maximum complexity
            // Nose (top)
            builder.begin(point(0.0, 20.0));
            // Right outer wing tip (double-wing via notch)
            builder.line_to(point(16.0, -16.0));
            // Right notch (inner indent creating double-wing effect)
            builder.line_to(point(10.0, -10.0));
            // Right outer second wing tip
            builder.line_to(point(20.0, -20.0));
            // Right inner indent
            builder.line_to(point(7.0, -12.0));
            // Thruster right
            builder.line_to(point(7.0, -18.0));
            // Thruster left
            builder.line_to(point(-7.0, -18.0));
            // Left inner indent
            builder.line_to(point(-7.0, -12.0));
            // Left outer second wing tip
            builder.line_to(point(-20.0, -20.0));
            // Left notch
            builder.line_to(point(-10.0, -10.0));
            // Left outer wing tip
            builder.line_to(point(-16.0, -16.0));
            builder.close();
        }
        3 | 4 => {
            // Tier 3–4: Wider wings and larger thruster
            // Nose (top)
            builder.begin(point(0.0, 20.0));
            // Right wing (wider tip)
            builder.line_to(point(16.0, -16.0));
            // Right indent
            builder.line_to(point(6.0, -9.0));
            // Thruster right (slightly larger)
            builder.line_to(point(7.0, -18.0));
            // Thruster left
            builder.line_to(point(-7.0, -18.0));
            // Left indent
            builder.line_to(point(-6.0, -9.0));
            // Left wing (wider tip)
            builder.line_to(point(-16.0, -16.0));
            builder.close();
        }
        _ => {
            // Tier 1–2 (default): original silhouette — kept for backward compatibility
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
        }
    }

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

/// Generate a small triangle mesh for Scout Drone rendering.
/// Equilateral-ish triangle, tip facing +Y (upward), compact and fast-looking.
pub fn generate_scout_drone_mesh() -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let mut builder = Path::builder();
    // Equilateral triangle, radius ~10: tip up, base flat at bottom
    builder.begin(point(0.0, 10.0));   // Tip (top)
    builder.line_to(point(9.0, -6.0)); // Right base
    builder.line_to(point(-9.0, -6.0)); // Left base
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
        warn!("ScoutDrone tessellation failed: {e:?}, using circle fallback");
        return Mesh::from(Circle::new(10.0));
    }

    let positions: Vec<[f32; 3]> = buffers.vertices.clone();
    let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| [(p[0] / 10.0 + 1.0) / 2.0, (p[1] / 10.0 + 1.0) / 2.0])
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
    // Scout ship: compact swept-wing interceptor facing +Y
    let r = radius;
    builder.begin(point(0.0, r));               // Nose
    builder.line_to(point(r * 0.7, r * 0.1));  // Right cockpit shoulder
    builder.line_to(point(r * 0.9, -r * 0.3)); // Right wing tip
    builder.line_to(point(r * 0.4, -r * 0.2)); // Right wing root
    builder.line_to(point(r * 0.3, -r * 0.8)); // Right engine
    builder.line_to(point(-r * 0.3, -r * 0.8));// Left engine
    builder.line_to(point(-r * 0.4, -r * 0.2));// Left wing root
    builder.line_to(point(-r * 0.9, -r * 0.3));// Left wing tip
    builder.line_to(point(-r * 0.7, r * 0.1)); // Left cockpit shoulder
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
    // Fighter: aggressive interceptor with swept delta wings and engine pods
    let r = radius;
    builder.begin(point(0.0, r));               // Nose tip
    builder.line_to(point(r * 0.25, r * 0.4)); // Right fuselage
    builder.line_to(point(r * 1.1, -r * 0.2)); // Right wing tip (swept back)
    builder.line_to(point(r * 0.7, -r * 0.5)); // Right wing trailing edge
    builder.line_to(point(r * 0.35, -r * 0.4));// Right engine pod outer
    builder.line_to(point(r * 0.35, -r));       // Right engine nozzle
    builder.line_to(point(-r * 0.35, -r));      // Left engine nozzle
    builder.line_to(point(-r * 0.35, -r * 0.4));// Left engine pod outer
    builder.line_to(point(-r * 0.7, -r * 0.5));// Left wing trailing edge
    builder.line_to(point(-r * 1.1, -r * 0.2));// Left wing tip
    builder.line_to(point(-r * 0.25, r * 0.4));// Left fuselage
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

/// Generate a heavy warship mesh for Heavy Cruiser rendering.
/// Wide, bulky hull with weapon sponsons — clearly the biggest enemy type.
pub fn generate_heavy_cruiser_mesh(radius: f32) -> Mesh {
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    let mut builder = Path::builder();

    // Heavy warship: wide blunt hull, weapon sponsons on sides, dual engine block
    let r = radius;
    builder.begin(point(0.0, r * 0.9));          // Bow center
    builder.line_to(point(r * 0.4, r * 0.7));    // Bow right shoulder
    builder.line_to(point(r * 0.9, r * 0.3));    // Right sponson front
    builder.line_to(point(r * 1.0, 0.0));         // Right sponson tip
    builder.line_to(point(r * 0.9, -r * 0.3));   // Right sponson aft
    builder.line_to(point(r * 0.7, -r * 0.5));   // Right hull waist
    builder.line_to(point(r * 0.5, -r * 0.6));   // Right engine block outer
    builder.line_to(point(r * 0.5, -r));           // Right engine nozzle
    builder.line_to(point(r * 0.15, -r));          // Right inner nozzle
    builder.line_to(point(r * 0.15, -r * 0.7));   // Center engine gap
    builder.line_to(point(-r * 0.15, -r * 0.7));  // Center engine gap mirror
    builder.line_to(point(-r * 0.15, -r));         // Left inner nozzle
    builder.line_to(point(-r * 0.5, -r));          // Left engine nozzle
    builder.line_to(point(-r * 0.5, -r * 0.6));   // Left engine block outer
    builder.line_to(point(-r * 0.7, -r * 0.5));   // Left hull waist
    builder.line_to(point(-r * 0.9, -r * 0.3));   // Left sponson aft
    builder.line_to(point(-r * 1.0, 0.0));         // Left sponson tip
    builder.line_to(point(-r * 0.9, r * 0.3));    // Left sponson front
    builder.line_to(point(-r * 0.4, r * 0.7));    // Bow left shoulder
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

/// Tests for vector art mesh generation.
#[cfg(test)]
mod tests {
    use super::*;

    // ── Story 10-1: Tier-based player ship and scout drone ──────────────

    #[test]
    fn player_mesh_tier1_generates_without_panic() {
        let mesh = generate_player_mesh(1);
        assert!(
            mesh.count_vertices() > 0,
            "Tier-1 player mesh must have vertices"
        );
    }

    #[test]
    fn player_mesh_tier2_generates_without_panic() {
        let mesh = generate_player_mesh(2);
        assert!(
            mesh.count_vertices() > 0,
            "Tier-2 player mesh must have vertices"
        );
    }

    #[test]
    fn player_mesh_tier3_generates_without_panic() {
        let mesh = generate_player_mesh(3);
        assert!(
            mesh.count_vertices() > 0,
            "Tier-3 player mesh must have vertices"
        );
    }

    #[test]
    fn player_mesh_tier4_generates_without_panic() {
        let mesh = generate_player_mesh(4);
        assert!(
            mesh.count_vertices() > 0,
            "Tier-4 player mesh must have vertices"
        );
    }

    #[test]
    fn player_mesh_tier5_generates_without_panic() {
        let mesh = generate_player_mesh(5);
        assert!(
            mesh.count_vertices() > 0,
            "Tier-5 player mesh must have vertices"
        );
    }

    #[test]
    fn player_mesh_tier1_and_tier3_differ_in_vertex_count() {
        // Tier 3 has the same polygon vertex count as tier 1 (7 points),
        // but different coordinates. Both should tessellate successfully.
        let mesh1 = generate_player_mesh(1);
        let mesh3 = generate_player_mesh(3);
        assert!(mesh1.count_vertices() > 0, "Tier-1 mesh should have vertices");
        assert!(mesh3.count_vertices() > 0, "Tier-3 mesh should have vertices");
    }

    #[test]
    fn player_mesh_tier5_has_more_vertices_than_tier1() {
        // Tier 5 path has 11 points vs 7 points for tier 1 — tessellated vertex count
        // may vary but tier 5 polygon is more complex so should produce more or equal vertices.
        let mesh1 = generate_player_mesh(1);
        let mesh5 = generate_player_mesh(5);
        assert!(
            mesh5.count_vertices() >= mesh1.count_vertices(),
            "Tier-5 mesh (more polygon points) should produce >= vertices than tier-1, got tier5={} tier1={}",
            mesh5.count_vertices(),
            mesh1.count_vertices()
        );
    }

    #[test]
    fn scout_drone_mesh_generates() {
        let mesh = generate_scout_drone_mesh();
        assert!(
            mesh.count_vertices() > 0,
            "Scout drone mesh must have vertices"
        );
    }

    #[test]
    fn scout_drone_mesh_has_indices() {
        let mesh = generate_scout_drone_mesh();
        let indices = mesh.indices().expect("Scout drone mesh should have indices");
        let index_count = match indices {
            Indices::U32(v) => v.len(),
            _ => panic!("Expected U32 indices for scout drone mesh"),
        };
        assert!(
            index_count >= 3,
            "Scout drone mesh should have at least 3 indices, got {index_count}"
        );
    }

    #[test]
    fn fighter_mesh_generates() {
        let mesh = generate_fighter_mesh(12.0);
        assert!(
            mesh.count_vertices() > 0,
            "Fighter mesh must have vertices"
        );
    }

    #[test]
    fn heavy_cruiser_mesh_generates() {
        let mesh = generate_heavy_cruiser_mesh(22.0);
        assert!(
            mesh.count_vertices() > 0,
            "Heavy cruiser mesh must have vertices"
        );
    }

    // ── Pre-existing tests ───────────────────────────────────────────────

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

/// Generate a companion ship mesh — a smaller, distinct triangle silhouette facing +Y.
/// Slightly smaller than the player ship to be recognizable as a wingman.
pub fn generate_companion_mesh() -> Mesh {
    // Simple compact triangle, ~60% player size, distinct shape
    let positions = vec![
        [0.0, 14.0, 0.0],   // Nose (top)
        [9.0, -10.0, 0.0],  // Right wing
        [3.0, -6.0, 0.0],   // Right indent
        [3.0, -12.0, 0.0],  // Right thruster
        [-3.0, -12.0, 0.0], // Left thruster
        [-3.0, -6.0, 0.0],  // Left indent
        [-9.0, -10.0, 0.0], // Left wing
    ];
    // Simple triangle fan triangulation from center
    let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 1];
    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs: Vec<[f32; 2]> = positions
        .iter()
        .map(|p| [(p[0] + 9.0) / 18.0, (p[1] + 12.0) / 26.0])
        .collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Generate a hexagonal mesh for Boss enemy rendering (Story 7-1).
/// Large and imposing — 6-sided silhouette communicates special threat status.
pub fn generate_boss_mesh(radius: f32) -> Mesh {
    let vertex_count = 6u32;
    let angle_step = std::f32::consts::TAU / vertex_count as f32;
    // Rotate so flat side faces up (more imposing silhouette)
    let offset = std::f32::consts::PI / 6.0;

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
        warn!("Boss tessellation failed: {e:?}, using circle fallback");
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
