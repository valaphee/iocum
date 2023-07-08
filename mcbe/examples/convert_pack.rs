use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fs::File,
};

use glam::{Vec2, Vec3};

use iokum_mcbe::{
    behavior_pack::{
        block,
        block::{MaterialInstance, RenderMethod},
    },
    pack::{Data, VersionedData},
    resource_pack::geometry,
};
use iokum_mcje::resource_pack::{block as src_block, model};

fn main() {
    let mut models = HashSet::new();
    for dir_entry in
        std::fs::read_dir(r#"C:\Users\valaphee\Downloads\assets\cb\blockstates\"#).unwrap()
    {
        let dir_entry = dir_entry.unwrap();
        if !dir_entry.metadata().unwrap().is_file() {
            continue;
        }
        let key = dir_entry
            .file_name()
            .to_str()
            .unwrap()
            .strip_suffix(".json")
            .unwrap()
            .to_string();

        println!("Block: {}", dir_entry.file_name().into_string().unwrap());
        block::Block {
            description: block::Description {
                identifier: key,
                properties: Default::default(),
                menu_category: None,
            },
            components: vec![],
            permutations: match serde_json::from_reader::<_, src_block::Block>(
                File::open(dir_entry.path()).unwrap(),
            )
            .unwrap()
            {
                src_block::Block::Variants(variants) => variants
                    .into_iter()
                    .map(|(variant_key, variant)| {
                        let model = variant
                            .0
                            .into_iter()
                            .max_by_key(|model| model.weight)
                            .unwrap();
                        models.insert(model.model);
                        block::Permutation {
                            condition: if variant_key.is_empty() {
                                "".to_string()
                            } else {
                                variant_key
                                    .split(',')
                                    .map(|key_value| {
                                        let (key, value) = key_value.split_once('=').unwrap();
                                        format!(
                                            "query.block_property('{}') == {}",
                                            key,
                                            match value {
                                                "false" => "false".to_string(),
                                                "true" => "true".to_string(),
                                                value => value
                                                    .parse::<u32>()
                                                    .map_or(format!("'{}'", value), |_| value
                                                        .to_string()),
                                            }
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" && ")
                            },
                            components: vec![block::Component::Transformation {
                                translation: [0.0, 0.0, 0.0],
                                scale: [0.0, 0.0, 0.0],
                                rotation: [model.x as f32, 360.0 - model.y as f32, 0.0],
                            }],
                        }
                    })
                    .collect(),
                src_block::Block::Multipart(multipart) => multipart
                    .into_iter()
                    .map(|case| {
                        let model = case
                            .apply
                            .into_iter()
                            .max_by_key(|model| model.weight)
                            .unwrap();
                        models.insert(model.model);
                        block::Permutation {
                            condition: case.when.map_or("".to_string(), |when| match when {
                                src_block::When::One(key_value) => {
                                    let (key, value) = key_value.into_iter().next().unwrap();
                                    format!(
                                        "query.block_property('{}') == {}",
                                        key,
                                        match value.as_str() {
                                            "false" => "false".to_string(),
                                            "true" => "true".to_string(),
                                            value => value
                                                .parse::<u32>()
                                                .map_or(format!("'{}'", value), |_| value
                                                    .to_string()),
                                        }
                                    )
                                }
                                src_block::When::Many(key_value) => {
                                    let (key, value) = key_value.into_iter().next().unwrap();
                                    value
                                        .into_iter()
                                        .map(|key_value| {
                                            let (key, value) =
                                                key_value.into_iter().next().unwrap();
                                            format!(
                                                "query.block_property('{}') == {}",
                                                key,
                                                match value.as_str() {
                                                    "false" => "false".to_string(),
                                                    "true" => "true".to_string(),
                                                    value => value
                                                        .parse::<u32>()
                                                        .map_or(format!("'{}'", value), |_| value
                                                            .to_string()),
                                                }
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                        .join(match key.as_str() {
                                            "AND" => " && ",
                                            "OR" => " || ",
                                            _ => todo!(),
                                        })
                                }
                            }),
                            components: vec![block::Component::Transformation {
                                translation: [0.0, 0.0, 0.0],
                                scale: [0.0, 0.0, 0.0],
                                rotation: [model.x as f32, 360.0 - model.y as f32, 0.0],
                            }],
                        }
                    })
                    .collect(),
            },
        };
    }

    let mut components_by_model = HashMap::new();
    let mut geometry_by_model = HashMap::new();
    'models: for model_key in models {
        let mut textures = HashMap::new();
        let mut geometry_key = String::new();
        let mut geometry_elements = vec![];
        let mut geometry_groups = vec![];
        let mut parent = Some(model_key.clone());
        while let Some(ref parent_key) = parent {
            let (namespace, key) = parent_key.split_once(':').unwrap();
            let Ok(file) = File::open(format!(
                r#"C:\Users\valaphee\Downloads\assets\{}\models\{}.json"#,
                namespace, key
            )) else {
                continue 'models;
            };

            println!("Model: {}", parent_key);
            let model: model::Model = serde_json::from_reader(file).unwrap();
            for (key, value) in model.textures {
                match textures.entry(key) {
                    Entry::Vacant(entry) => {
                        entry.insert(value);
                    }
                    _ => {}
                }
            }
            if geometry_key.is_empty() && !model.elements.is_empty() {
                geometry_key = format!("geometry.{}", parent_key.rsplit('/').next().unwrap());
                if !geometry_by_model.contains_key(&geometry_key) {
                    geometry_elements = model.elements;
                    geometry_groups = model.groups;
                }
            }
            parent = model.parent.clone();
        }
        components_by_model.insert(
            model_key.rsplit('/').next().unwrap().to_string(),
            vec![
                block::Component::Geometry {
                    identifier: geometry_key.clone(),
                    bone_visibility: Default::default(),
                },
                block::Component::MaterialInstances(
                    textures
                        .into_iter()
                        .map(|(texture_key, texture)| {
                            (
                                texture_key.strip_prefix('#').unwrap().to_string(),
                                MaterialInstance {
                                    ambient_occlusion: true,
                                    face_dimming: true,
                                    render_method: RenderMethod::Opaque,
                                    texture,
                                },
                            )
                        })
                        .collect(),
                ),
            ],
        );
        if !geometry_elements.is_empty() {
            geometry_by_model.insert(geometry_key, (geometry_elements, geometry_groups));
        }
    }

    for (geometry_key, (elements, groups)) in geometry_by_model {
        // generate list of bones and create references to element ids
        let mut bones = vec![geometry::Bone {
            name: "".to_string(),
            parent: None,
            pivot: None,
            rotation: None,
            mirror: None,
            inflate: None,
            cubes: vec![],
        }];
        let mut bone_origins = vec![Vec3::ZERO];
        let mut bone_id_by_element_id = HashMap::new();
        let mut groups_ = vec![];
        for group in groups {
            groups_.push((0, group));
        }
        while let Some((bone_id, group)) = groups_.pop() {
            match group {
                model::Group::Group {
                    name,
                    origin,
                    children,
                } => {
                    let bone_id = bones.len();
                    bones.push(geometry::Bone {
                        name,
                        parent: None,
                        pivot: Some(origin),
                        rotation: None,
                        mirror: None,
                        inflate: None,
                        cubes: vec![],
                    });
                    bone_origins.push(origin);
                    for child in children {
                        groups_.push((bone_id, child));
                    }
                }
                model::Group::Element(element_id) => {
                    bone_id_by_element_id.insert(element_id, bone_id);
                }
            }
        }

        // add cubes to bones
        for (element_id, element) in elements.into_iter().enumerate() {
            let bone_id = *bone_id_by_element_id
                .get(&(element_id as u32))
                .unwrap_or(&0);
            let rotation;
            let pivot;
            if let Some(element_rotation) = element.rotation {
                rotation = Some(match element_rotation.axis {
                    model::Axis::X => Vec3::new(element_rotation.angle, 0.0, 0.0),
                    model::Axis::Y => Vec3::new(0.0, element_rotation.angle, 0.0),
                    model::Axis::Z => Vec3::new(0.0, 0.0, element_rotation.angle),
                });
                // element_rotation.origin.x + (element.to.x - element.from.x)
                pivot = Some(Vec3::new(-8.0, 0.0, -8.0) + element_rotation.origin);
            } else {
                rotation = None;
                pivot = None;
            }
            bones[bone_id].cubes.push(geometry::Cube {
                origin: Some(Vec3::new(-8.0, 0.0, -8.0) + element.from),
                size: Some(element.to - element.from),
                rotation,
                pivot,
                inflate: None,
                mirror: None,
                uv: element
                    .faces
                    .into_iter()
                    .map(|(face_key, face)| {
                        let uv;
                        let uv_size;
                        if let Some(uv_from_to) = face.uv {
                            uv = Vec2::new(uv_from_to[0], uv_from_to[1]);
                            uv_size = Some(Vec2::new(
                                uv_from_to[2] - uv_from_to[0],
                                uv_from_to[3] - uv_from_to[1],
                            ));
                        } else {
                            uv = Vec2::new(0.0, 0.0);
                            uv_size = None;
                        }
                        (
                            match face_key {
                                model::FaceKey::North => geometry::FaceKey::North,
                                model::FaceKey::South => geometry::FaceKey::South,
                                model::FaceKey::East => geometry::FaceKey::East,
                                model::FaceKey::West => geometry::FaceKey::West,
                                model::FaceKey::Up => geometry::FaceKey::Up,
                                model::FaceKey::Down => geometry::FaceKey::Down,
                            },
                            geometry::Face {
                                uv,
                                uv_size,
                                material_instance: Some(
                                    face.texture.strip_prefix("#").unwrap().to_string(),
                                ),
                            },
                        )
                    })
                    .collect(),
            });
        }

        serde_json::to_writer_pretty(
            File::create(format!(
                r#"C:\Users\valaphee\Downloads\resource_pack\models\entity\{}.geo.json"#,
                geometry_key
            ))
            .unwrap(),
            &VersionedData {
                format_version: "1.16.0".to_string(),
                data: Data::Geometry(vec![geometry::Geometry {
                    description: geometry::Description {
                        identifier: format!("geometry.{}", geometry_key),
                        visible_bounds_width: None,
                        visible_bounds_height: None,
                        visible_bounds_offset: None,
                        texture_width: None,
                        texture_height: None,
                    },
                    bones,
                }]),
            },
        )
        .unwrap();
    }
}
