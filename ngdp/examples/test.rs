use std::{collections::HashMap, time::Instant};

use iocum_ngdp::{casc::Storage, tact::Encoding};

fn main() {
    let storage = Storage::new("/drive_c/Program Files (x86)/Overwatch/data/casc/data").unwrap();
    let encoding = Encoding::decode(
        &mut storage
            .get(&hex::decode("141501e28e197283065ac405b461b9d8").unwrap())
            .unwrap()
            .unwrap()
            .as_slice(),
    )
    .unwrap();
    let x = Instant::now();
    let e_key = encoding
        .get(&hex::decode("0354005609a6edf3faf02aa305b58e44").unwrap())
        .unwrap();
    println!("{} {}", hex::encode(e_key), x.elapsed().as_millis());
}
