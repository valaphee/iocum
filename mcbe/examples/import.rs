use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    path::{Path, PathBuf},
};

use glam::{Vec2, Vec3};

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

    blocks: HashMap<String, blocks::Block>,
    components: HashMap<String, Vec<block::Component>>,
    geometries: HashMap<String, Vec<model::Element>>,
    textures: HashMap<String, block::RenderMethod>,
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
            vanilla_texture_atlas: serde_json::from_reader(File::open(r"C:\Program Files\WindowsApps\Microsoft.MinecraftUWP_1.20.102.0_x64__8wekyb3d8bbwe\data\resource_packs\vanilla\textures\terrain_texture.json").unwrap()).unwrap(),
            asset_path: asset_path.as_ref().to_path_buf(),
            behavior_pack_path: behavior_pack_path.as_ref().to_path_buf(),
            resource_pack_path: resource_pack_path.as_ref().to_path_buf(),
            blocks: Default::default(),
            components: Default::default(),
            geometries: Default::default(),
            textures: Default::default(),
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
                    // collect properties (only when not known beforehand)
                    if !variant_state.is_empty() {
                        for property in variant_state.split(',') {
                            let (key, value) = property.split_once('=').unwrap();
                            match block
                                .description
                                .properties
                                .entry(format!("{}:{}", namespace, key))
                            {
                                Entry::Occupied(mut entry) => match entry.get_mut() {
                                    block::Property::Bool(values) => {
                                        let value = value.parse().unwrap();
                                        if !values.contains(&value) {
                                            values.push(value)
                                        }
                                    }
                                    block::Property::Int(values) => {
                                        let value = value.parse().unwrap();
                                        if !values.contains(&value) {
                                            values.push(value)
                                        }
                                    }
                                    block::Property::Enum(values) => {
                                        let value = value.to_owned();
                                        if !values.contains(&value) {
                                            values.push(value)
                                        }
                                    }
                                    _ => unreachable!(),
                                },
                                Entry::Vacant(entry) => {
                                    entry.insert(if let Ok(value) = value.parse::<bool>() {
                                        block::Property::Bool(vec![value])
                                    } else if let Ok(value) = value.parse::<u32>() {
                                        block::Property::Int(vec![value])
                                    } else {
                                        block::Property::Enum(vec![value.to_owned()])
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

                    // either add to components or add new permutation
                    if variant_state.is_empty() {
                        block.components.append(&mut components);
                    } else {
                        let condition = variant_state
                            .split(',')
                            .map(|property| {
                                let (key, value) = property.split_once('=').unwrap();
                                format!(
                                    "query.block_property('{}:{}') == {}",
                                    namespace,
                                    key,
                                    match value {
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

        // sort property values (only when not known beforehand)
        for property in block.description.properties.values_mut() {
            match property {
                block::Property::Bool(values) => {
                    values.sort();
                }
                block::Property::Int(values) => {
                    *property = block::Property::IntRange {
                        values: block::Range {
                            min: *values.iter().min().unwrap(),
                            max: *values.iter().max().unwrap(),
                        },
                    }
                }
                block::Property::Enum(values) => {
                    values.sort();
                }
                _ => unreachable!(),
            }
        }

        // remove empty permutations
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
        let mut ambient_occlusion = None;
        let mut textures = HashMap::new();
        let mut geometry = String::new();
        let mut geometry_elements = vec![];
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
            if ambient_occlusion.is_none() {
                ambient_occlusion = Some(model.ambient_occlusion)
            }
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
                    geometry_elements = model.elements;
                }
            }

            let Some(next_parent) = model.parent.clone() else {
                break;
            };
            parent = next_parent;
        }

        // save geometry
        if !geometry_elements.is_empty() {
            // check if it's a unit cube and use built-in model
            if geometry_elements.len() == 1 {
                if let Some(block_key) = block {
                    let element = geometry_elements.first().unwrap();
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

            self.geometries.insert(geometry.clone(), geometry_elements);
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
                                ambient_occlusion: ambient_occlusion.unwrap(),
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

    fn write_geometries(&self) {
        println!("Writing geometries...");
        for (geometry_key, elements) in &self.geometries {
            println!("Writing geometry: {}", geometry_key);
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

            // write geometry
            serde_json::to_writer_pretty(
                File::create(
                    self.resource_pack_path
                        .join(format!("models/entity/{}.geo.json", geometry_key)),
                )
                .unwrap(),
                &VersionedData {
                    format_version: "1.16.0".to_owned(),
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

    fn write_textures(&self) {
        println!("Writing textures...");

        // write terrain textures
        serde_json::to_writer_pretty(
            File::create(
                self.resource_pack_path
                    .join("textures/terrain_texture.json"),
            )
            .unwrap(),
            &self.texture_atlas,
        )
        .unwrap();

        // write flipbook textures
        serde_json::to_writer_pretty(
            File::create(
                self.resource_pack_path
                    .join("textures/flipbook_textures.json"),
            )
            .unwrap(),
            &self.flipbook_textures,
        )
        .unwrap();
    }

    fn write_blocks(&self) {
        println!("Writing blocks...");
        serde_json::to_writer_pretty(
            File::create(self.resource_pack_path.join("blocks.json")).unwrap(),
            &blocks::Blocks {
                format_version: "1.19.30".to_owned(),
                blocks: self.blocks.clone(),
            },
        )
        .unwrap();
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
            .to_owned();
        let block = importer.import_blockstate(format!("cb:{}", key));

        // write block
        serde_json::to_writer_pretty(
            File::create(
                importer
                    .behavior_pack_path
                    .join(format!(r"blocks\{}.json", key)),
            )
            .unwrap(),
            &VersionedData {
                format_version: "1.20.0".to_owned(),
                data: Data::Block(block),
            },
        )
        .unwrap();
    }

    importer.write_geometries();
    importer.write_textures();
    importer.write_blocks();
}
