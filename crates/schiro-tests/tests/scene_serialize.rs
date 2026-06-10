/// Tests for the SceneFile serialization format used by the editor
/// save/load feature. The structs are duplicated from
/// `schiro_editor::scene` so that the test crate does not pull in
/// the full editor dependency graph.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SceneFile {
    version: u32,
    entities: Vec<EntityDesc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntityDesc {
    name: String,
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
    rotator: Option<[f32; 3]>,
    mesh: Option<MeshDesc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum MeshDesc {
    Sphere { segments: u32, rings: u32 },
    Grid { rows: u32, cols: u32, spacing: f32 },
    Cube,
    Plane,
}

const SAMPLE_JSON: &str = r#"{
  "version": 1,
  "entities": [
    {
      "name": "Sphere",
      "translation": [0.0, 1.5, 0.0],
      "rotation": [0.0, 0.0, 0.0, 1.0],
      "scale": [1.0, 1.0, 1.0],
      "rotator": [0.0, 1.5, 0.0],
      "mesh": {
        "kind": "Sphere",
        "segments": 32,
        "rings": 16
      }
    },
    {
      "name": "Light",
      "translation": [0.0, 3.0, 0.0],
      "rotation": [0.0, 0.0, 0.0, 1.0],
      "scale": [1.0, 1.0, 1.0],
      "rotator": null,
      "mesh": null
    }
  ]
}"#;

#[test]
fn json_roundtrip_default_scene() {
    let file: SceneFile = serde_json::from_str(SAMPLE_JSON).expect("deserialize");
    assert_eq!(file.version, 1);
    assert_eq!(file.entities.len(), 2);

    let sphere = &file.entities[0];
    assert_eq!(sphere.name, "Sphere");
    assert_eq!(sphere.translation, [0.0, 1.5, 0.0]);
    assert_eq!(sphere.rotator, Some([0.0, 1.5, 0.0]));
    assert!(sphere.mesh.is_some());

    let light = &file.entities[1];
    assert_eq!(light.name, "Light");
    assert!(light.mesh.is_none());
}

#[test]
fn json_serialize_then_deserialize_is_stable() {
    let original = SceneFile {
        version: 1,
        entities: vec![
            EntityDesc {
                name: "Cube".into(),
                translation: [-1.0, 2.0, 0.5],
                rotation: [0.0, 0.707, 0.0, 0.707],
                scale: [2.0, 2.0, 2.0],
                rotator: None,
                mesh: Some(MeshDesc::Cube),
            },
            EntityDesc {
                name: "Plane".into(),
                translation: [0.0, -2.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
                rotator: Some([0.0, 2.0, 0.0]),
                mesh: Some(MeshDesc::Plane),
            },
            EntityDesc {
                name: "Empty".into(),
                translation: [5.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
                rotator: None,
                mesh: None,
            },
        ],
    };

    let json = serde_json::to_string_pretty(&original).expect("serialize");
    let round: SceneFile = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(round.version, original.version);
    assert_eq!(round.entities.len(), original.entities.len());

    for (a, b) in original.entities.iter().zip(round.entities.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.translation, b.translation);
        assert_eq!(a.rotation, b.rotation);
        assert_eq!(a.scale, b.scale);
        assert_eq!(a.rotator, b.rotator);
        match (&a.mesh, &b.mesh) {
            (Some(a), Some(b)) => match (a, b) {
                (
                    MeshDesc::Sphere { segments: s1, rings: r1 },
                    MeshDesc::Sphere { segments: s2, rings: r2 },
                ) => {
                    assert_eq!(s1, s2);
                    assert_eq!(r1, r2);
                }
                (
                    MeshDesc::Grid { rows: r1, cols: c1, spacing: s1 },
                    MeshDesc::Grid { rows: r2, cols: c2, spacing: s2 },
                ) => {
                    assert_eq!(r1, r2);
                    assert_eq!(c1, c2);
                    assert!((s1 - s2).abs() < 1e-6);
                }
                (MeshDesc::Cube, MeshDesc::Cube) => {}
                (MeshDesc::Plane, MeshDesc::Plane) => {}
                _ => panic!("mesh kind changed during round-trip"),
            },
            (None, None) => {}
            _ => panic!("mesh presence changed during round-trip"),
        }
    }
}

#[test]
fn json_empty_scene_is_valid() {
    let file = SceneFile { version: 1, entities: vec![] };
    let json = serde_json::to_string(&file).expect("serialize empty");
    let round: SceneFile = serde_json::from_str(&json).expect("deserialize empty");
    assert_eq!(round.version, 1);
    assert!(round.entities.is_empty());
}

#[test]
fn json_unknown_mesh_tag_fails_gracefully() {
    let bad = r#"{"version":1,"entities":[{"name":"x","translation":[0,0,0],"rotation":[0,0,0,1],"scale":[1,1,1],"rotator":null,"mesh":{"kind":"Unknown"}}]}"#;
    let result: Result<SceneFile, _> = serde_json::from_str(bad);
    assert!(result.is_err(), "unknown mesh tag should fail");
}

#[test]
fn json_version_mismatch_is_detectable() {
    let future = r#"{"version":99,"entities":[]}"#;
    let file: SceneFile = serde_json::from_str(future).expect("deserialize");
    assert_eq!(file.version, 99);
}
