use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fs::File,
    path::{Path, PathBuf},
};

use glam::{Vec2, Vec3};

use iokum_mcbe::{
    behavior_pack::{
        block,
        block::{MaterialInstance, Property, RenderMethod},
    },
    pack::{Data, VersionedData},
    resource_pack::{
        flipbook_textures, geometry, texture_atlas,
    },
};
use iokum_mcje::resource_pack::{block_state, mcmeta, model};

struct Importer {
    asset_path: PathBuf,
    behavior_pack_path: PathBuf,
    resource_pack_path: PathBuf,

    components: HashMap<String, Vec<block::Component>>,
    geometries: HashMap<String, Vec<model::Element>>,
    textures: HashSet<String>,
}

impl Importer {
    fn new(
        asset_path: impl AsRef<Path>,
        behavior_pack_path: impl AsRef<Path>,
        resource_pack_path: impl AsRef<Path>,
    ) -> Self {
        Self {
            asset_path: asset_path.as_ref().to_path_buf(),
            behavior_pack_path: behavior_pack_path.as_ref().to_path_buf(),
            resource_pack_path: resource_pack_path.as_ref().to_path_buf(),
            geometries: Default::default(),
            components: Default::default(),
            textures: Default::default(),
        }
    }

    fn import_blockstate(&mut self, blockstate: String) -> block::Block {
        let (namespace, key) = blockstate.split_once(':').unwrap();
        println!("Block: {}", blockstate);
        let mut block = block::Block {
            description: block::Description {
                identifier: format!("cb:{}", key),
                properties: Default::default(),
                menu_category: block::MenuCategory {
                    category: block::Category::Construction,
                    group: None,
                },
            },
            components: vec![],
            permutations: vec![],
        };
        match serde_json::from_reader::<_, block_state::BlockState>(
            File::open(
                self.asset_path
                    .join(format!(r"{}\blockstates\{}.json", namespace, key)),
            )
            .unwrap(),
        )
        .unwrap()
        {
            block_state::BlockState::Variants(variants) => {
                for (variant_key, variant) in variants {
                    // collect properties (unordered)
                    if !variant_key.is_empty() {
                        for key_value in variant_key.split(',') {
                            let (key, value) = key_value.split_once('=').unwrap();
                            match block.description.properties.entry(format!("cb:{}", key)) {
                                Entry::Occupied(mut entry) => match entry.get_mut() {
                                    Property::Bool(values) => {
                                        let value = value.parse().unwrap();
                                        if !values.contains(&value) {
                                            values.push(value)
                                        }
                                    }
                                    Property::Int(values) => {
                                        let value = value.parse().unwrap();
                                        if !values.contains(&value) {
                                            values.push(value)
                                        }
                                    }
                                    Property::Enum(values) => {
                                        let value = value.to_string();
                                        if !values.contains(&value) {
                                            values.push(value)
                                        }
                                    }
                                    _ => unreachable!(),
                                },
                                Entry::Vacant(entry) => {
                                    entry.insert(if let Ok(value) = value.parse::<bool>() {
                                        Property::Bool(vec![value])
                                    } else if let Ok(value) = value.parse::<u32>() {
                                        Property::Int(vec![value])
                                    } else {
                                        Property::Enum(vec![value.to_string()])
                                    });
                                }
                            }
                        }
                    }

                    // import model and create components
                    let model = variant
                        .0
                        .into_iter()
                        .max_by_key(|model| model.weight)
                        .unwrap();
                    let mut components = self.import_model(model.model);
                    if model.x != 0 || model.y != 0 {
                        components.push(block::Component::Transformation {
                            translation: Vec3::ZERO,
                            scale: Vec3::ONE,
                            rotation: Vec3::new(model.x as f32, model.y as f32, 0.0),
                        });
                    }

                    // either set components or add to permutations
                    if variant_key.is_empty() {
                        block.components = components;
                    } else {
                        let condition = variant_key
                            .split(',')
                            .map(|key_value| {
                                let (key, value) = key_value.split_once('=').unwrap();
                                format!(
                                    "query.block_property('cb:{}') == {}",
                                    key,
                                    match value {
                                        "false" => "false".to_string(),
                                        "true" => "true".to_string(),
                                        value => value
                                            .parse::<u32>()
                                            .map_or(format!("'{}'", value), |_| {
                                                value.to_string()
                                            }),
                                    }
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" && ");
                        block.permutations.push(block::Permutation {
                            condition,
                            components,
                        });
                    }
                }
            }
            block_state::BlockState::Multipart(multipart) => for _case in multipart {},
        };
        block
    }

    fn import_model(&mut self, model: String) -> Vec<block::Component> {
        // return cached model
        if let Some(components) = self.components.get(&model) {
            return components.clone();
        }

        // merge all models
        let mut ambient_occlusion = None;
        let mut textures = HashMap::new();
        let mut geometry_key = String::new();
        let mut geometry_elements = vec![];
        let mut parent_ = Some(model.clone());
        while let Some(parent) = &parent_ {
            let (namespace, key) = parent.split_once(':').unwrap();
            let Ok(file) = File::open(
                self.asset_path
                    .join(format!(r"{}\models\{}.json", namespace, key)),
            ) else {
                return vec![];
            };

            println!("Model: {}", parent);
            let model: model::Model = serde_json::from_reader(file).unwrap();
            if ambient_occlusion.is_none() {
                ambient_occlusion = Some(model.ambient_occlusion)
            }
            for (key, value) in model.textures {
                if let Entry::Vacant(entry) = textures.entry(key) {
                    entry.insert(value);
                }
            }
            if geometry_key.is_empty() && !model.elements.is_empty() {
                geometry_key = parent.rsplit('/').next().unwrap().to_string();
                if !self.geometries.contains_key(&geometry_key) {
                    geometry_elements = model.elements;
                }
            }
            parent_ = model.parent.clone();
        }

        // set default texture and remove textures which are the same as the default
        let mut default_texture = textures
            .remove("particle")
            .unwrap_or_else(|| textures.values().next().unwrap().to_string());
        if let Some(particle_texture_key) = default_texture.strip_prefix('#') {
            default_texture = textures.get(particle_texture_key).unwrap().clone();
        }
        let mut textures_to_remove = vec![];
        for (texture_key, texture) in &textures {
            if texture == &default_texture {
                textures_to_remove.push(texture_key.clone());
            }
        }
        for texture_key in textures_to_remove {
            textures.remove(&texture_key);
        }
        textures.insert("*".to_string(), default_texture);

        // save geometry
        if !geometry_elements.is_empty() {
            self.geometries
                .insert(geometry_key.clone(), geometry_elements);
        }

        // save components
        let components = vec![
            block::Component::Geometry {
                identifier: format!("geometry.{}", geometry_key),
                bone_visibility: Default::default(),
            },
            block::Component::MaterialInstances(
                textures
                    .into_iter()
                    .map(|(texture_key, texture)| {
                        let texture_name = texture.rsplit('/').next().unwrap().to_string();
                        self.textures.insert(texture);
                        (
                            texture_key,
                            MaterialInstance {
                                ambient_occlusion: ambient_occlusion.unwrap(),
                                face_dimming: true,
                                render_method: RenderMethod::Opaque,
                                texture: texture_name,
                            },
                        )
                    })
                    .collect(),
            ),
        ];
        self.components.insert(model, components.clone());

        components
    }

    fn write_textures(&self) {
        let mut terrain_texture = texture_atlas::TextureAtlas {
            num_mip_levels: 0,
            padding: 0,
            resource_pack_name: "cb".to_string(),
            texture_data: Default::default(),
            texture_name: "atlas.terrain".to_string(),
        };
        let mut flipbook_textures: Vec<flipbook_textures::FlipbookTexture> = vec![];
        for texture in &self.textures {
            let (namespace, key) = texture
                .split_once(':')
                .unwrap_or(("minecraft", texture.as_str()));
            let texture_name = texture.rsplit('/').next().unwrap().to_string();
            let texture_path = format!("textures/blocks/{}.png", texture_name);
            std::fs::copy(
                self.asset_path
                    .join(format!("{}/textures/{}.png", namespace, key)),
                self.resource_pack_path.join(texture_path.clone()),
            )
            .unwrap();

            // read mcmeta
            let mcmeta_path = self
                .asset_path
                .join(format!("{}/textures/{}.png.mcmeta", namespace, key));
            if mcmeta_path.exists() {
                let mcmeta =
                    serde_json::from_reader::<_, mcmeta::McMeta>(File::open(mcmeta_path).unwrap())
                        .unwrap();
                if let Some(animation) = mcmeta.animation {
                    // add to flipbook textures
                    flipbook_textures.push(flipbook_textures::FlipbookTexture {
                        flipbook_texture: texture_path.clone(),
                        atlas_index: None,
                        atlas_tile_variant: None,
                        atlas_tile: texture_name.clone(),
                        ticks_per_frame: animation.frametime,
                        frames: vec![],
                        replicate: 1,
                        blend_frames: animation.interpolate,
                    })
                }
            }

            // add to terrain textures
            terrain_texture.texture_data.insert(
                texture_name,
                texture_atlas::TextureData {
                    textures: texture_atlas::Texture {
                        overlay_color: None,
                        path: texture_path,
                        tint_color: None,
                        variations: vec![],
                    },
                },
            );
        }
        serde_json::to_writer_pretty(
            File::create(
                self.resource_pack_path
                    .join("textures/terrain_texture.json"),
            )
            .unwrap(),
            &terrain_texture,
        )
        .unwrap();
        serde_json::to_writer_pretty(
            File::create(
                self.resource_pack_path
                    .join("textures/flipbook_textures.json"),
            )
            .unwrap(),
            &flipbook_textures,
        )
        .unwrap();
    }

    fn write_geometries(&self) {
        for (geometry_key, elements) in &self.geometries {
            // generate list of bones and create references to element ids
            let mut bone = geometry::Bone {
                name: geometry_key.clone(),
                parent: None,
                pivot: None,
                rotation: None,
                mirror: None,
                inflate: None,
                cubes: vec![],
            };

            // add cubes to bones
            for element in elements {
                let rotation;
                let pivot;
                if let Some(element_rotation) = &element.rotation {
                    rotation = Some(match element_rotation.axis {
                        model::Axis::X => Vec3::new(-element_rotation.angle, 0.0, 0.0),
                        model::Axis::Y => Vec3::new(0.0, -element_rotation.angle, 0.0),
                        model::Axis::Z => Vec3::new(0.0, 0.0, element_rotation.angle),
                    });
                    pivot = Some(Vec3::new(
                        -element_rotation.origin.x + 8.0,
                        element_rotation.origin.y,
                        element_rotation.origin.z - 8.0,
                    ));
                } else {
                    rotation = None;
                    pivot = None;
                }
                bone.cubes.push(geometry::Cube {
                    origin: Some(Vec3::new(
                        -element.to.x + 8.0,
                        element.from.y,
                        element.from.z - 8.0,
                    )),
                    size: Some(element.to - element.from),
                    rotation,
                    pivot,
                    inflate: None,
                    mirror: None,
                    uv: element
                        .faces
                        .iter()
                        .map(|(&face_key, face)| {
                            let uv;
                            let uv_size;
                            if let Some(uv_from_to) = face.uv {
                                uv = Vec2::new(uv_from_to[0], uv_from_to[1]);
                                uv_size = Vec2::new(
                                    uv_from_to[2] - uv_from_to[0],
                                    uv_from_to[3] - uv_from_to[1],
                                );
                            } else {
                                todo!()
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
                                    uv_size: Some(uv_size),
                                    material_instance: Some(
                                        face.texture.strip_prefix('#').unwrap().to_string(),
                                    ),
                                },
                            )
                        })
                        .collect(),
                });
            }

            serde_json::to_writer_pretty(
                File::create(
                    self.resource_pack_path
                        .join(format!("models/entity/{}.geo.json", geometry_key)),
                )
                .unwrap(),
                &VersionedData {
                    format_version: "1.16.0".to_string(),
                    data: Data::Geometry(vec![geometry::Geometry {
                        description: geometry::Description {
                            identifier: format!("geometry.{}", geometry_key),
                            visible_bounds_width: None,
                            visible_bounds_height: None,
                            visible_bounds_offset: None,
                            texture_width: Some(16),
                            texture_height: Some(16),
                        },
                        bones: vec![bone],
                    }]),
                },
            )
            .unwrap();
        }
    }
}

fn main() {
    let mut importer = Importer::new(
        r"C:\Users\valaphee\Downloads\assets",
        r"C:\Users\valaphee\AppData\Local\Packages\Microsoft.MinecraftUWP_8wekyb3d8bbwe\LocalState\games\com.mojang\development_behavior_packs\MysteryMod",
        r"C:\Users\valaphee\AppData\Local\Packages\Microsoft.MinecraftUWP_8wekyb3d8bbwe\LocalState\games\com.mojang\development_resource_packs\MysteryMod",
    );

    for dir_entry in std::fs::read_dir(importer.asset_path.join("cb/blockstates")).unwrap() {
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
        let block = importer.import_blockstate(format!("cb:{}", key));
        serde_json::to_writer_pretty(
            File::create(
                importer
                    .behavior_pack_path
                    .join(format!(r"blocks\{}.json", key)),
            )
            .unwrap(),
            &VersionedData {
                format_version: "1.19.30".to_string(),
                data: Data::Block(block),
            },
        )
        .unwrap();
    }

    importer.write_textures();
    importer.write_geometries();
}
