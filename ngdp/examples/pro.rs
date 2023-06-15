use std::{fs::File, path::PathBuf};
use std::collections::HashMap;

use iocum_ngdp::{
    casc::Storage,
    tact::{BuildConfig, BuildInfo, Encoding},
};
use iocum_ngdp::tact::RootFile;

fn main() {
    let path = PathBuf::from("...");
    let storage = Storage::new(path.join("data/casc/data")).unwrap();
    for build_info in BuildInfo::parse(&mut File::open(path.join(".build.info")).unwrap())
        .unwrap() {
        let build_config = BuildConfig::parse(
            &mut File::open(path.join(format!(
                "data/casc/config/{:02x}/{:02x}/{:016x}",
                (build_info.build_config >> 120) as u8,
                (build_info.build_config >> 112) as u8,
                build_info.build_config
            )))
                .unwrap(),
        )
            .unwrap();
        let encoding = Encoding::decode(
            &mut storage
                .get(build_config.encoding.e_key.unwrap())
                .unwrap()
                .unwrap()
                .as_slice(),
        )
            .unwrap();
        let mut root_files = HashMap::new();
        for root_file in RootFile::parse(&mut storage
            .get(encoding.get(build_config.root.c_key).unwrap())
            .unwrap()
            .unwrap()
            .as_slice()
        ).unwrap() {
            root_files.insert(root_file.id, root_file.md5);
        }
    }
}
