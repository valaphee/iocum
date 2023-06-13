use serde::Deserialize;

use crate::guid::Guid;

#[derive(Deserialize, Debug)]
pub struct MapHeader {
    #[serde(rename = "0342E00E")]
    pub map_intro: MapIntro,
    #[serde(rename = "364ADD04")]
    pub _364add04: Guid,
    #[serde(rename = "44D13CC2")]
    pub _44d13cc2: u32,
    #[serde(rename = "45283CE2")]
    pub _45283ce2: Vec<u64>,
    #[serde(rename = "4E03E9C6")]
    pub _4e03e9c6: Option<Vec<_2DDBBB42>>,
    #[serde(rename = "4E87690F")]
    pub _4e87690f: Guid,
    #[serde(rename = "506FA8D8")]
    pub map_name: String,
    #[serde(rename = "53DFAF05")]
    pub _53dfaf05: Vec<u64>,
    #[serde(rename = "5AFE2F61")]
    pub _5afe2f61: Option<Guid>,
    #[serde(rename = "5DB91CE2")]
    pub _5db91ce2: Guid,
    #[serde(rename = "78715D57")]
    pub _78715d57: Vec<_7FB10A24>,
    #[serde(rename = "8EBADA44")]
    pub _8ebada44: Option<Guid>,
    #[serde(rename = "9289BADC")]
    pub _9289badc: Option<Vec<_2DDBBB42>>,
    #[serde(rename = "C0CA4671")]
    pub _c0ca4671: Vec<_71B2D30A>,
    #[serde(rename = "D97BC44F")]
    pub _d97bc44f: Vec<_71B2D30A>,
}

#[derive(Deserialize, Debug)]
pub struct MapIntro {
    #[serde(rename = "86C1CFAB")]
    pub loading_screen: Option<Guid>,
    #[serde(rename = "9386E669")]
    pub small_map_icon: Option<Guid>,
    #[serde(rename = "956158FF")]
    pub announcer_welcome: Guid,
    #[serde(rename = "A0AE2E3E")]
    pub music_tease: Guid,
    #[serde(rename = "C6599DEB")]
    pub loading_screen_flag: Guid,
    #[serde(rename = "D978BBDC")]
    pub _d978bbdc: Option<Guid>,
}

#[derive(Deserialize, Debug)]
pub struct _2DDBBB42 {
    #[serde(rename = "321C3BCE")]
    pub _321c3bce: u64,
    #[serde(rename = "BA53D5ED")]
    pub _ba53d5ed: u64,
}

#[derive(Deserialize, Debug)]
pub struct _7FB10A24 {
    #[serde(rename = "0342E00E")]
    pub map_intro: MapIntro,
    #[serde(rename = "2A9103F4")]
    pub _2a9103f4: Guid,
    #[serde(rename = "364ADD04")]
    pub _364add04: Guid,
    #[serde(rename = "38BFB46C")]
    pub _38bfb46c: Guid,
    #[serde(rename = "BF231F12")]
    pub _bf231f12: u64,
}

#[derive(Deserialize, Debug)]
pub struct _71B2D30A {
    #[serde(rename = "A9253C68")]
    pub _a9253c68: Guid,
    #[serde(rename = "EB4F2408")]
    pub gamemode: Guid,
}
