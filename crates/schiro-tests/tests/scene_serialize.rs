/// Tests for the SceneFile serialization format.  The types now
/// live in `schiro_scene` so the test crate can import them
/// directly without duplicating struct definitions.
use schiro_scene::{EntityDesc, MeshDesc, SceneFile};

const SAMPLE_JSON: &str = r#"{
  "version": 1,
  "entities": [
    {
      "name": "Sphere",
      "translation": [0.0, 1.5, 0.0],
      "rotation": [0.0, 0.0, 0.0, 1.0],
      "scale": [1.0, 1.0, 1.0],
      "rotator": [0.0, 1.5, 0.0],
      "mesh": {"kind":"Sphere","segments":32,"rings":16}
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
    assert!(file.entities[0].mesh.is_some());
    assert!(file.entities[1].mesh.is_none());
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
    assert_eq!(round.entities.len(), 3);

    for (a, b) in original.entities.iter().zip(round.entities.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.translation, b.translation);
        assert_eq!(a.rotation, b.rotation);
        assert_eq!(a.rotator, b.rotator);
    }
}

#[test]
fn json_empty_scene_is_valid() {
    let file = SceneFile { version: 1, entities: vec![] };
    let json = serde_json::to_string(&file).unwrap();
    let round: SceneFile = serde_json::from_str(&json).unwrap();
    assert!(round.entities.is_empty());
}

#[test]
fn json_unknown_mesh_tag_fails_gracefully() {
    let bad = r#"{"version":1,"entities":[{"name":"x","translation":[0,0,0],"rotation":[0,0,0,1],"scale":[1,1,1],"rotator":null,"mesh":{"kind":"Unknown"}}]}"#;
    assert!(serde_json::from_str::<SceneFile>(bad).is_err());
}

#[test]
fn json_version_mismatch_is_detectable() {
    let future = r#"{"version":99,"entities":[]}"#;
    let file: SceneFile = serde_json::from_str(future).unwrap();
    assert_eq!(file.version, 99);
}
