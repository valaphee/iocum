use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    path::{Path, PathBuf},
};
use std::collections::BTreeMap;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use serde_with::{KeyValueMap, serde_as};

use iokum_mcbe::{
    behavior_pack::block,
    pack::{Data, VersionedData},
    resource_pack::{blocks, flipbook_textures, geometry, texture_atlas},
};
use iokum_mcje::resource_pack::{block_state, mcmeta, model};

struct Importer {
    texture_mappings: HashMap<String, HashMap<String, String>>,
    vanilla_texture_atlas: texture_atlas::TextureAtlas,

    asset_path: PathBuf,
    behavior_pack_path: PathBuf,
    resource_pack_path: PathBuf,

    // cache
    components: HashMap<String, Vec<block::Component>>,
    textures: HashMap<String, block::RenderMethod>,

    blocks: HashMap<String, blocks::Block>,
    geometries: HashMap<String, geometry::Geometry>,
    texture_atlas: texture_atlas::TextureAtlas,
    flipbook_textures: Vec<flipbook_textures::FlipbookTexture>,
}

impl Importer {
    fn new(
        asset_path: impl AsRef<Path>,
        behavior_pack_path: impl AsRef<Path>,
        resource_pack_path: impl AsRef<Path>,
    ) -> Self {
        Self {
            texture_mappings: serde_json::from_reader(File::open("textures.json").unwrap()).unwrap(),
            vanilla_texture_atlas: serde_json::from_reader(File::open(r"C:\Program Files\WindowsApps\Microsoft.MinecraftUWP_1.20.1001.0_x64__8wekyb3d8bbwe\data\resource_packs\vanilla\textures\terrain_texture.json").unwrap()).unwrap(),
            asset_path: asset_path.as_ref().to_path_buf(),
            behavior_pack_path: behavior_pack_path.as_ref().to_path_buf(),
            resource_pack_path: resource_pack_path.as_ref().to_path_buf(),
            components: Default::default(),
            textures: Default::default(),
            blocks: Default::default(),
            geometries: Default::default(),
            texture_atlas: texture_atlas::TextureAtlas {
                resource_pack_name: "cb".to_string(),
                texture_name: "atlas.terrain".to_string(),
                padding: 0,
                num_mip_levels: 0,
                texture_data: Default::default(),
            },
            flipbook_textures: vec![],
        }
    }

    fn import_blockstate(&mut self, blockstate: String) -> block::Block {
        println!("Importing block: {}", blockstate);
        let (namespace, key) = blockstate.split_once(':').unwrap();
        let mut block = block::Block {
            description: block::Description {
                identifier: blockstate.clone(),
                properties: Default::default(),
                menu_category: block::MenuCategory {
                    category: block::Category::Construction,
                    group: None,
                },
                traits: vec![],
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
                // TODO: only useful for blocks with one model
                let first_model = variants
                    .values()
                    .next()
                    .unwrap()
                    .0
                    .iter()
                    .max_by_key(|model| model.weight)
                    .unwrap();
                let single_model = variants.values().all(|variant| {
                    first_model.model
                        == variant
                            .0
                            .iter()
                            .max_by_key(|model| model.weight)
                            .unwrap()
                            .model
                });
                if single_model {
                    block.components = self.import_model(
                        first_model.model.to_owned(),
                        if variants.len() == 1 && first_model.x == 0 && first_model.y == 0 {
                            Some(block.description.identifier.clone())
                        } else {
                            None
                        },
                    );
                }

                for (variant_state, variant) in variants {
                    let condition = variant_state
                        .split(',')
                        .map(|property| {
                            let Some((key, value)) = property.split_once('=') else {
                                return "".to_string();
                            };

                            format!(
                                "query.block_property('{}:{}') == {}",
                                namespace,
                                key,
                                match value {
                                    "false" => "false".to_owned(),
                                    "true" => "true".to_owned(),
                                    value => value
                                        .parse::<u32>()
                                        .map_or(format!("'{}'", value), |_| { value.to_owned() }),
                                }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" && ");

                    // import model and create components
                    let model = variant
                        .0
                        .into_iter()
                        .max_by_key(|model| model.weight)
                        .unwrap();
                    let mut components = if single_model {
                        vec![]
                    } else {
                        self.import_model(model.model, None)
                    };
                    if model.x != 0 || model.y != 0 {
                        components.push(block::Component::Transformation {
                            translation: Vec3::ZERO,
                            scale: Vec3::ONE,
                            rotation: Vec3::new(model.x as f32, model.y as f32, 0.0),
                        });
                    }

                    // add new permutation
                    block.permutations.push(block::Permutation {
                        condition,
                        components,
                    });
                }
            }
            block_state::BlockState::Multipart(multipart) => {
                for case in multipart {
                    let condition = case.when.map_or("".to_string(), |when| match when {
                        block_state::When::One(property) => {
                            let (key, value) = property.into_iter().next().unwrap();

                            format!(
                                "query.block_property('{}:{}') == {}",
                                namespace,
                                key,
                                match value.as_str() {
                                    "false" => "false".to_owned(),
                                    "true" => "true".to_owned(),
                                    value => value
                                        .parse::<u32>()
                                        .map_or(format!("'{}'", value), |_| { value.to_owned() }),
                                }
                            )
                        }
                        block_state::When::Many(properties) => {
                            let (key, value) = properties.into_iter().next().unwrap();
                            value
                                .into_iter()
                                .map(|property| {
                                    let (key, value) = property.into_iter().next().unwrap();

                                    format!(
                                        "query.block_property('{}:{}') == {}",
                                        namespace,
                                        key,
                                        match value.as_str() {
                                            "false" => "false".to_owned(),
                                            "true" => "true".to_owned(),
                                            value => value
                                                .parse::<u32>()
                                                .map_or(format!("'{}'", value), |_| {
                                                    value.to_owned()
                                                }),
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
                    });

                    // import model and create components
                    let model = case
                        .apply
                        .into_iter()
                        .max_by_key(|model| model.weight)
                        .unwrap();
                    let mut components = self.import_model(model.model, None);
                    if model.x != 0 || model.y != 0 {
                        components.push(block::Component::Transformation {
                            translation: Vec3::ZERO,
                            scale: Vec3::ONE,
                            rotation: Vec3::new(model.x as f32, model.y as f32, 0.0),
                        });
                    }

                    // add new permutation
                    block.permutations.push(block::Permutation {
                        condition,
                        components,
                    });
                }
            }
        };

        // set default components and remove empty permutations
        if let Some(position) = block.permutations.iter().position(|permutation| permutation.condition.is_empty()) {
            block.components.append(&mut block.permutations.remove(position).components);
        }
        block
            .permutations
            .retain(|permutation| !permutation.components.is_empty());

        block
    }

    fn import_model(&mut self, model: String, block: Option<String>) -> Vec<block::Component> {
        // return cached model
        if let Some(components) = self.components.get(&model) {
            return components.clone();
        }

        // merge all models
        let mut textures = HashMap::new();
        let mut geometry = String::new();
        let mut elements = vec![];
        let mut parent = model.clone();
        loop {
            let (namespace, key) = parent.split_once(':').unwrap();
            let Ok(file) = File::open(
                self.asset_path
                    .join(format!(r"{}\models\{}.json", namespace, key)),
            ) else {
                return vec![];
            };

            println!("Importing model: {}", parent);
            let model: model::Model = serde_json::from_reader(file).unwrap();
            for (texture_ref, texture) in model.textures {
                if let Entry::Vacant(entry) = textures.entry(texture_ref) {
                    entry.insert(if texture.starts_with('#') || texture.contains(':') {
                        texture
                    } else {
                        format!("minecraft:{}", texture)
                    });
                }
            }
            if geometry.is_empty() && !model.elements.is_empty() {
                geometry = Self::sanitize(&parent);
                if !self.geometries.contains_key(&geometry) {
                    elements = model.elements;
                }
            }

            let Some(next_parent) = model.parent.clone() else {
                break;
            };
            parent = next_parent;
        }

        // save geometry
        if !elements.is_empty() {
            // check if it's a unit cube and use built-in model
            if elements.len() == 1 {
                if let Some(block_key) = block {
                    let element = elements.first().unwrap();
                    if element.from == Vec3::ZERO && element.to == Vec3::new(16.0, 16.0, 16.0) {
                        let faces = &element.faces;
                        let (north, north_render_method) = self.import_texture(
                            textures
                                .get(
                                    faces
                                        .get(&model::FaceEnum::North)
                                        .unwrap()
                                        .texture
                                        .strip_prefix('#')
                                        .unwrap(),
                                )
                                .unwrap(),
                        );
                        let (south, south_render_method) = self.import_texture(
                            textures
                                .get(
                                    faces
                                        .get(&model::FaceEnum::South)
                                        .unwrap()
                                        .texture
                                        .strip_prefix('#')
                                        .unwrap(),
                                )
                                .unwrap(),
                        );
                        let (east, east_render_method) = self.import_texture(
                            textures
                                .get(
                                    faces
                                        .get(&model::FaceEnum::East)
                                        .unwrap()
                                        .texture
                                        .strip_prefix('#')
                                        .unwrap(),
                                )
                                .unwrap(),
                        );
                        let (west, west_render_method) = self.import_texture(
                            textures
                                .get(
                                    faces
                                        .get(&model::FaceEnum::West)
                                        .unwrap()
                                        .texture
                                        .strip_prefix('#')
                                        .unwrap(),
                                )
                                .unwrap(),
                        );
                        let (up, up_render_method) = self.import_texture(
                            textures
                                .get(
                                    faces
                                        .get(&model::FaceEnum::Up)
                                        .unwrap()
                                        .texture
                                        .strip_prefix('#')
                                        .unwrap(),
                                )
                                .unwrap(),
                        );
                        let (down, down_render_method) = self.import_texture(
                            textures
                                .get(
                                    faces
                                        .get(&model::FaceEnum::Down)
                                        .unwrap()
                                        .texture
                                        .strip_prefix('#')
                                        .unwrap(),
                                )
                                .unwrap(),
                        );
                        if north_render_method == block::RenderMethod::Opaque
                            && south_render_method == block::RenderMethod::Opaque
                            && east_render_method == block::RenderMethod::Opaque
                            && west_render_method == block::RenderMethod::Opaque
                            && up_render_method == block::RenderMethod::Opaque
                            && down_render_method == block::RenderMethod::Opaque
                        {
                            self.blocks.insert(
                                block_key,
                                blocks::Block {
                                    isotropic: None,
                                    textures: Some(
                                        if north == south && north == east && north == west {
                                            if north == up && up == down {
                                                blocks::Face::CubeAll(north)
                                            } else {
                                                blocks::Face::CubeBottomTop {
                                                    up,
                                                    down,
                                                    side: north,
                                                }
                                            }
                                        } else {
                                            blocks::Face::Cube {
                                                up,
                                                down,
                                                north,
                                                south,
                                                east,
                                                west,
                                            }
                                        },
                                    ),
                                    carried_textures: None,
                                    brightness_gamma: 1.0,
                                    sound: None,
                                },
                            );
                            return vec![];
                        }
                    }
                }
            }

            let mut bone = geometry::Bone {
                name: geometry.clone(),
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
                        .map(|(&face_enum, face)| {
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
                                match face_enum {
                                    model::FaceEnum::North => geometry::FaceEnum::North,
                                    model::FaceEnum::South => geometry::FaceEnum::South,
                                    model::FaceEnum::East => geometry::FaceEnum::East,
                                    model::FaceEnum::West => geometry::FaceEnum::West,
                                    model::FaceEnum::Up => geometry::FaceEnum::Up,
                                    model::FaceEnum::Down => geometry::FaceEnum::Down,
                                },
                                geometry::Face {
                                    uv,
                                    uv_size: Some(uv_size),
                                    material_instance: Some(
                                        face.texture.strip_prefix('#').unwrap().to_owned(),
                                    ),
                                },
                            )
                        })
                        .collect(),
                });
            }

            self.geometries.insert(
                geometry.clone(),
                geometry::Geometry {
                    description: geometry::Description {
                        identifier: format!("geometry.{}", geometry),
                        visible_bounds_width: None,
                        visible_bounds_height: None,
                        visible_bounds_offset: None,
                        texture_width: Some(16),
                        texture_height: Some(16),
                    },
                    bones: vec![bone],
                },
            );
        }

        // set default texture, remove textures which are the same as the default,
        // import textures and select most fitting render method
        let mut default_texture = textures
            .remove("particle")
            .unwrap_or_else(|| textures.values().next().unwrap().to_owned());
        if let Some(particle_texture_ref) = default_texture.strip_prefix('#') {
            default_texture = textures.get(particle_texture_ref).unwrap().clone();
        }
        let mut render_method = block::RenderMethod::Opaque;
        let mut textures_to_remove = vec![];
        for (texture_ref, texture) in &mut textures {
            if texture == &default_texture {
                textures_to_remove.push(texture_ref.clone());
            }

            let (imported_texture, imported_render_method) = self.import_texture(texture);
            *texture = imported_texture;
            if render_method == block::RenderMethod::Opaque
                && (imported_render_method == block::RenderMethod::Blend
                    || imported_render_method == block::RenderMethod::AlphaTest)
                || render_method == block::RenderMethod::AlphaTest
                    && imported_render_method == block::RenderMethod::Blend
            {
                render_method = imported_render_method;
            }
        }
        for texture_ref in textures_to_remove {
            textures.remove(&texture_ref);
        }
        textures.insert("*".to_owned(), self.import_texture(&default_texture).0);

        // save components
        let components = vec![
            block::Component::Geometry {
                identifier: format!("geometry.{}", geometry),
                bone_visibility: Default::default(),
            },
            block::Component::MaterialInstances(
                textures
                    .into_iter()
                    .map(|(texture_ref, texture)| {
                        (
                            texture_ref,
                            block::MaterialInstance {
                                ambient_occlusion: true,
                                face_dimming: true,
                                render_method,
                                texture,
                            },
                        )
                    })
                    .collect(),
            ),
        ];
        self.components.insert(model, components.clone());

        components
    }

    fn import_texture(&mut self, texture: &str) -> (String, block::RenderMethod) {
        if let Some(render_method) = self.textures.get(texture) {
            return (Self::sanitize(texture), *render_method);
        }

        // check if it's a vanilla texture
        let (namespace, key) = texture.split_once(':').unwrap();
        if namespace == "minecraft" {
            let (dir, file) = key.split_once('/').unwrap();
            if dir == "block" {
                let texture_path = format!("textures/blocks/{}", &self.texture_mappings[dir][file]);
                if let Some(texture_name) = self
                    .vanilla_texture_atlas
                    .texture_data
                    .iter()
                    .find(|(_, texture)| {
                        if texture.textures.len() != 1 {
                            return false;
                        }
                        texture.textures[0].path == texture_path
                    })
                    .map(|(texture_name, _)| texture_name)
                {
                    // select render method based on texture
                    let image = image::io::Reader::open(
                        self.asset_path
                            .join(format!("{}/textures/{}.png", namespace, key)),
                    )
                    .unwrap()
                    .decode()
                    .unwrap()
                    .to_rgba8();
                    let mut render_method = block::RenderMethod::Opaque;
                    for pixel in image.pixels() {
                        let alpha = pixel.0[3];
                        if alpha != 0xFF {
                            if alpha != 0x00 {
                                render_method = block::RenderMethod::Blend;
                                break;
                            } else {
                                render_method = block::RenderMethod::AlphaTest;
                            }
                        }
                    }

                    return (texture_name.to_owned(), render_method);
                };
            }
        }

        println!("Importing texture: {}", texture);
        let texture_name = Self::sanitize(texture);
        let texture_path = format!("textures/blocks/{}", texture_name);

        // select render method based on texture
        let image = image::io::Reader::open(
            self.asset_path
                .join(format!("{}/textures/{}.png", namespace, key)),
        )
        .unwrap()
        .decode()
        .unwrap()
        .to_rgba8();
        let mut render_method = block::RenderMethod::Opaque;
        for pixel in image.pixels() {
            let alpha = pixel.0[3];
            if alpha != 0xFF {
                if alpha != 0x00 {
                    render_method = block::RenderMethod::Blend;
                    break;
                } else {
                    render_method = block::RenderMethod::AlphaTest;
                }
            }
        }

        // copy texture
        std::fs::copy(
            self.asset_path
                .join(format!("{}/textures/{}.png", namespace, key)),
            self.resource_pack_path
                .join(format!("{}.png", texture_path)),
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
                println!("+ Animation");
                // add to flipbook textures
                self.flipbook_textures
                    .push(flipbook_textures::FlipbookTexture {
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

        // add to terrain textures and save render method
        self.texture_atlas.texture_data.insert(
            texture_name.clone(),
            texture_atlas::TextureData {
                textures: vec![texture_atlas::Texture {
                    overlay_color: None,
                    path: texture_path,
                    tint_color: None,
                    variations: vec![],
                }],
            },
        );
        self.textures.insert(texture_name.clone(), render_method);

        (texture_name, render_method)
    }

    fn sanitize(name: &str) -> String {
        name.split_once(':')
            .unwrap()
            .1
            .split_once('/')
            .unwrap()
            .1
            .replace('/', "_")
    }
}

fn main() {
    let mut importer = Importer::new(
        r"C:\Users\valaphee\Downloads\assets",
        r"C:\Users\valaphee\AppData\Local\Packages\Microsoft.MinecraftUWP_8wekyb3d8bbwe\LocalState\games\com.mojang\development_behavior_packs\MysteryMod",
        r"C:\Users\valaphee\AppData\Local\Packages\Microsoft.MinecraftUWP_8wekyb3d8bbwe\LocalState\games\com.mojang\development_resource_packs\MysteryMod",
    );

    let blocks: ExportBlocks = serde_json::from_reader(File::open(r"C:\Users\valaphee\CLionProjects\iokum\blocks.json").unwrap()).unwrap();
    let mappingtail: HashMap<String, Vec<String>> = serde_json::from_reader(File::open(r"C:\Users\valaphee\CLionProjects\iokum\mappingtail.json").unwrap()).unwrap();
    let mappingtail_blockstates = mappingtail.get("blockstates").unwrap();
    let mut blockstail = Vec::new();
    let mut itemstail = Vec::new();
    for block_data in blocks.0 {
        let mut block = importer.import_blockstate(block_data.id.clone());

        let mut properties = BTreeMap::new();
        for property in block_data.properties {
            let mut property_values = property.second.into_iter();
            let first_value = property_values.next().unwrap();
            let mut new_property = if let Ok(value) = first_value.parse::<bool>() {
                block::Property::Bool(vec![value])
            } else if let Ok(value) = first_value.parse::<u32>() {
                block::Property::Int(vec![value])
            } else {
                block::Property::Enum(vec![first_value.to_owned()])
            };
            for value in property_values {
                match &mut new_property {
                    block::Property::Bool(values) => {
                        values.push(value.parse().unwrap())
                    }
                    block::Property::Int(values) => {
                        values.push(value.parse().unwrap())
                    }
                    block::Property::Enum(values) => {
                        values.push(value)
                    }
                    _ => unreachable!()
                }
            }
            properties.insert(format!("cb:{}", property.first), new_property);
        }
        block.description.properties = properties;
        {
            let shape = block_data.collision_shape;
            let min = Vec3::new(shape.min_x, shape.min_y, shape.min_z).clamp(Vec3::ZERO, Vec3::ONE);
            let max = Vec3::new(shape.max_x, shape.max_y, shape.max_z).clamp(Vec3::ZERO, Vec3::ONE);
            let size = max - min;
            if size.x == 0.0 || size.y == 0.0 || size.z == 0.0 {
                block.components.push(block::Component::CollisionBox(block::BoolOrValue::Bool(false)));
            } else if min != Vec3::ZERO || max != Vec3::ONE {
                block.components.push(block::Component::CollisionBox(block::BoolOrValue::Value(block::BoundingBox {
                    origin: (min - Vec3::new(0.5, 0.0, 0.5)) * 16.0,
                    size: size * 16.0,
                })));
            }
        }
        if block_data.blast_resistance >= 0.0 && block_data.blast_resistance <= 3600000.0 {
            block.components.push(block::Component::DestructibleByExplosion(block::BoolOrValue::Value(block::DestructibleByExplosion {
                explosion_resistance: block_data.blast_resistance,
            })));
        } else {
            block.components.push(block::Component::DestructibleByExplosion(block::BoolOrValue::Bool(false)));
        }
        if block_data.hardness >= 0.0 {
            block.components.push(block::Component::DestructibleByMining(block::BoolOrValue::Value(block::DestructibleByMining {
                seconds_to_destroy: block_data.hardness,
            })));
        } else {
            block.components.push(block::Component::DestructibleByMining(block::BoolOrValue::Bool(false)));
        }
        if block_data.burnable {
            block.components.push(block::Component::Flammable(block::BoolOrValue::Bool(true)));
        }
        if block_data.opacity != 0 {
            block.components.push(block::Component::LightDampening(15));
        }
        if block_data.luminance != 0 {
            block.components.push(block::Component::LightEmission(block_data.luminance));
        }
        if block_data.map_color != 0 {
            block.components.push(block::Component::MapColor(format!("#{:06X}", block_data.map_color)));
        }
        {
            let shape = block_data.outline_shape;
            let min = Vec3::new(shape.min_x, shape.min_y, shape.min_z).clamp(Vec3::ZERO, Vec3::ONE);
            let max = Vec3::new(shape.max_x, shape.max_y, shape.max_z).clamp(Vec3::ZERO, Vec3::ONE);
            let size = max - min;
            if size.x == 0.0 || size.y == 0.0 || size.z == 0.0 {
                block.components.push(block::Component::SelectionBox(block::BoolOrValue::Bool(false)));
            } else if min != Vec3::ZERO || max != Vec3::ONE {
                block.components.push(block::Component::SelectionBox(block::BoolOrValue::Value(block::BoundingBox {
                    origin: (min - Vec3::new(0.5, 0.0, 0.5)) * 16.0,
                    size: size * 16.0,
                })));
            }
        }

        // write block
        serde_json::to_writer_pretty(
            File::create(
                importer
                    .behavior_pack_path
                    .join(format!(r"blocks\{}.json", block.description.identifier.split_once(':').unwrap().1)),
            )
            .unwrap(),
            &VersionedData {
                format_version: "1.20.0".to_owned(),
                data: Data::Block(block),
            },
        )
        .unwrap();

        for blockstate in mappingtail_blockstates.iter().filter(|blockstate| blockstate.starts_with(&block_data.id)) {
            let mut properties = BTreeMap::new();
            if blockstate.contains('[') {
                for property in blockstate[blockstate.find('[').unwrap() + 1..blockstate.len() - 1].split(',') {
                    let (key, value) = property.split_once('=').unwrap();
                    properties.insert(key.to_owned(), if let Ok(value) = value.parse::<bool>() {
                        GeyserBlockMappingProperty::Bool(value)
                    } else if let Ok(value) = value.parse::<u32>() {
                        GeyserBlockMappingProperty::Int(value)
                    } else {
                        GeyserBlockMappingProperty::Enum(value.to_owned())
                    });
                }
            }
            blockstail.push(GeyserBlockMapping {
                java_identifier: blockstate.clone(),
                bedrock_identifier: block_data.id.clone(),
                bedrock_states: properties,
            });
        }
        itemstail.push(GeyserItemMapping {
            java_identifier: block_data.id.clone(),
            bedrock_identifier: block_data.id,
            bedrock_data: 0,
        });
    }

    // write blocks
    serde_json::to_writer_pretty(
        File::create(importer.resource_pack_path.join("blocks.json")).unwrap(),
        &blocks::Blocks {
            format_version: "1.19.30".to_owned(),
            blocks: importer.blocks,
        },
    )
    .unwrap();

    // write geometries
    for (geometry_key, geometry) in importer.geometries {
        serde_json::to_writer_pretty(
            File::create(
                importer
                    .resource_pack_path
                    .join(format!("models/entity/{}.geo.json", geometry_key)),
            )
            .unwrap(),
            &VersionedData {
                format_version: "1.16.0".to_owned(),
                data: Data::Geometry(vec![geometry]),
            },
        )
        .unwrap();
    }

    // write texture list
    serde_json::to_writer_pretty(
        File::create(
            importer
                .resource_pack_path
                .join("textures/textures_list.json"),
        )
        .unwrap(),
        &importer
            .texture_atlas
            .texture_data
            .values()
            .flat_map(|textures| textures.textures.iter().map(|texture| &texture.path))
            .collect::<Vec<_>>(),
    )
    .unwrap();

    // write texture atlas
    serde_json::to_writer_pretty(
        File::create(
            importer
                .resource_pack_path
                .join("textures/terrain_texture.json"),
        )
        .unwrap(),
        &importer.texture_atlas,
    )
    .unwrap();

    // write flipbook textures
    serde_json::to_writer_pretty(
        File::create(
            importer
                .resource_pack_path
                .join("textures/flipbook_textures.json"),
        )
        .unwrap(),
        &importer.flipbook_textures,
    )
    .unwrap();

    // write blockmappings
    serde_json::to_writer_pretty(
        File::create("blockstail.json").unwrap(),
        &blockstail,
    )
    .unwrap();

    // write itemmappings
    serde_json::to_writer_pretty(
        File::create("itemstail.json").unwrap(),
        &itemstail,
    )
    .unwrap();
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
struct ExportBlocks(
    #[serde_as(as = "KeyValueMap<_>")]
    Vec<ExportBlock>
);

#[derive(Debug, Serialize, Deserialize)]
struct ExportBlock {
    #[serde(rename = "$key$")]
    id: String,
    luminance: u8,
    hardness: f32,
    map_color: u32,
    outline_shape: ExportBlockShape,
    collision_shape: ExportBlockShape,
    burnable: bool,
    opacity: u32,
    opaque_full_cube: bool,
    blast_resistance: f32,
    properties: Vec<ExportBlockProperty>
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportBlockProperty {
    first: String,
    second: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportBlockShape {
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeyserBlockMapping {
    java_identifier: String,
    bedrock_identifier: String,
    bedrock_states: BTreeMap<String, GeyserBlockMappingProperty>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GeyserBlockMappingProperty {
    Bool(bool),
    Int(u32),
    Enum(String)
}

#[derive(Debug, Serialize, Deserialize)]
struct GeyserItemMapping {
    java_identifier: String,
    bedrock_identifier: String,
    bedrock_data: u32,
}
