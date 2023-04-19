pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/bgs.protocol.rs"));

    pub mod v2 {
        include!(concat!(env!("OUT_DIR"), "/bgs.protocol.v2.rs"));
    }

    pub mod account {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.account.v1.rs"));
        }
    }

    pub mod authentication {
        pub mod v1 {
            include!(concat!(
                env!("OUT_DIR"),
                "/bgs.protocol.authentication.v1.rs"
            ));
        }
    }

    pub mod challenge {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.challenge.v1.rs"));
        }
    }

    pub mod channel {
        include!(concat!(env!("OUT_DIR"), "/bgs.protocol.channel.rs"));

        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.channel.v1.rs"));
        }

        pub mod v2 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.channel.v2.rs"));

            pub mod membership {
                include!(concat!(
                    env!("OUT_DIR"),
                    "/bgs.protocol.channel.v2.membership.rs"
                ));
            }
        }
    }

    pub mod club {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.club.v1.rs"));

            pub mod membership {
                include!(concat!(
                    env!("OUT_DIR"),
                    "/bgs.protocol.club.v1.membership.rs"
                ));
            }
        }
    }

    pub mod connection {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.connection.v1.rs"));
        }
    }

    pub mod diag {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.diag.v1.rs"));
        }
    }

    pub mod friends {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.friends.v1.rs"));
        }
    }

    pub mod game_utilities {
        pub mod v2 {
            pub mod client {
                include!(concat!(
                    env!("OUT_DIR"),
                    "/bgs.protocol.game_utilities.v2.client.rs"
                ));
            }
        }
    }

    pub mod presence {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.presence.v1.rs"));
        }
    }

    pub mod report {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.report.v1.rs"));
        }

        pub mod v2 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.report.v2.rs"));
        }
    }

    pub mod resources {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.resources.v1.rs"));
        }
    }

    pub mod session {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.session.v1.rs"));
        }
    }

    pub mod sns {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.sns.v1.rs"));
        }
    }

    pub mod user_manager {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.user_manager.v1.rs"));
        }
    }

    pub mod voice {
        pub mod v2 {
            pub mod client {
                include!(concat!(env!("OUT_DIR"), "/bgs.protocol.voice.v2.client.rs"));
            }
        }
    }

    pub mod whisper {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/bgs.protocol.whisper.v1.rs"));
        }
    }
}
