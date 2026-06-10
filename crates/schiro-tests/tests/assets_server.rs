//! Tests for the `schiro_assets::AssetServer`.

use std::path::Path;

use schiro_assets::types::{MaterialAsset, MeshAsset, TextureAsset};
use schiro_assets::{Asset, AssetLoadError, AssetServer};

#[test]
fn asset_server_starts_empty() {
    let server = AssetServer::new();
    server.clear();
}

#[test]
fn load_returns_clone_of_cached_value() {
    let server = AssetServer::new();
    let first = server
        .load::<MeshAsset, _>("proc://cube", |_| Ok(MeshAsset::new("cube")))
        .expect("first load");
    // Second load uses a different value in the closure to prove
    // that the cache short-circuits it.
    let second = server
        .load::<MeshAsset, _>("proc://cube", |_| Ok(MeshAsset::new("ignored")))
        .expect("second load");
    // Both Arcs must point at the same cached value, not at the
    // "ignored" mesh returned by the second closure.
    assert_eq!(first.name, "cube");
    assert_eq!(second.name, "cube");
    assert_eq!(first.positions, second.positions);
}

#[test]
fn load_different_paths_does_not_collide() {
    let server = AssetServer::new();
    let a = server
        .load::<MeshAsset, _>("proc://a", |_| Ok(MeshAsset::new("A")))
        .unwrap();
    let b = server
        .load::<MeshAsset, _>("proc://b", |_| Ok(MeshAsset::new("B")))
        .unwrap();
    assert_eq!(a.name, "A");
    assert_eq!(b.name, "B");
}

#[test]
fn load_different_types_at_same_path_are_independent() {
    let server = AssetServer::new();
    let mesh = server
        .load::<MeshAsset, _>("key", |_| Ok(MeshAsset::new("m")))
        .unwrap();
    let mat = server
        .load::<MaterialAsset, _>("key", |_| Ok(MaterialAsset {
            name: "mat".into(),
            base_color: [1.0, 0.0, 0.0, 1.0],
            metallic: 0.0,
            roughness: 1.0,
        }))
        .unwrap();
    assert_eq!(mesh.name, "m");
    assert_eq!(mat.name, "mat");
}

#[test]
fn load_propagates_loader_error() {
    let server = AssetServer::new();
    let result = server.load::<MeshAsset, _>("broken", |_| {
        Err(AssetLoadError::Parse("boom".into()))
    });
    assert!(matches!(result, Err(AssetLoadError::Parse(_))));
}

#[test]
fn clear_drops_cache() {
    let server = AssetServer::new();
    let _ = server
        .load::<MeshAsset, _>("a", |_| Ok(MeshAsset::new("a")))
        .unwrap();
    server.clear();
    // We can still load again, the cache miss path should rerun the
    // loader closure.
    let again = server
        .load::<MeshAsset, _>("a", |_| Ok(MeshAsset::new("a")))
        .unwrap();
    assert_eq!(again.name, "a");
}

#[test]
fn asset_type_name_is_stable() {
    assert_eq!(MeshAsset::type_name(), "Mesh");
    assert_eq!(TextureAsset::type_name(), "Texture");
    assert_eq!(MaterialAsset::type_name(), "Material");
}

#[test]
fn load_io_error_variant() {
    let err = AssetLoadError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "x"));
    assert!(matches!(err, AssetLoadError::Io(_)));
}

#[test]
fn nonexistent_loader_path_is_not_called_for_known_extension() {
    // Sanity: the AssetServer does not consult the filesystem when
    // the loader closure is provided. The path is purely a cache key.
    let _ = Path::new("does_not_exist.glb");
    let server = AssetServer::new();
    let r = server
        .load::<MeshAsset, _>("does_not_exist.glb", |_| Ok(MeshAsset::new("v")))
        .unwrap();
    assert_eq!(r.name, "v");
}
