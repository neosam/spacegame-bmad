//! Tests for Story 10-4: Atmospheric Background — nebula clouds and deep star layer.

#![deny(clippy::unwrap_used)]

use void_drifter::rendering::background::{NebulaConfig, StarfieldConfig};

#[test]
fn nebula_config_defaults() {
    let config = NebulaConfig::default();
    assert!(config.enabled, "Nebula should be enabled by default");
    assert!(
        config.base_alpha > 0.0,
        "base_alpha must be positive, got {}",
        config.base_alpha
    );
}

#[test]
fn starfield_config_has_four_layers() {
    let config = StarfieldConfig::default();
    assert_eq!(
        config.layers.len(),
        4,
        "StarfieldConfig should have exactly 4 layers (3 original + 1 deep layer)"
    );
}

#[test]
fn deep_star_layer_has_correct_properties() {
    let config = StarfieldConfig::default();
    let deep_layer = &config.layers[3];
    assert!(
        deep_layer.parallax_factor < 0.01,
        "Deep layer should have very small parallax factor, got {}",
        deep_layer.parallax_factor
    );
    assert!(
        deep_layer.star_radius < 0.5,
        "Deep layer should have very small stars, got {}",
        deep_layer.star_radius
    );
    assert!(
        deep_layer.brightness < 0.15,
        "Deep layer should be dim, got {}",
        deep_layer.brightness
    );
    assert!(
        deep_layer.z_depth < -12.0,
        "Deep layer should be behind other layers, got z={}",
        deep_layer.z_depth
    );
}
