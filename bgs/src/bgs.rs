use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
};

use byteorder::{BigEndian, WriteBytesExt};
use prost::Message;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

pub struct RemoteService {
    request_tx: UnboundedSender<Vec<u8>>,
    responders: Arc<Mutex<HashMap<u32, oneshot::Sender<Vec<u8>>>>>,

    next_token: AtomicU32,
}

impl RemoteService {
    pub fn new(request_tx: UnboundedSender<Vec<u8>>) -> Self {
        Self {
            request_tx,
            responders: Arc::new(Mutex::new(HashMap::new())),
            next_token: AtomicU32::new(0),
        }
    }

    pub fn request_no_response(
        &self,
        service_hash: u32,
        method_id: u32,
        request: impl Message,
    ) -> u32 {
        let header = protocol::Header {
            service_id: 0,
            method_id: Some(method_id),
            token: self.next_token.fetch_add(1, Ordering::SeqCst),
            object_id: None,
            size: Some(request.encoded_len() as u32),
            status: None,
            error: vec![],
            timeout: None,
            is_response: None,
            forward_targets: vec![],
            service_hash: Some(service_hash),
            client_id: None,
            fanout_target: vec![],
            client_id_fanout_target: vec![],
            client_record: None,
            original_sender: None,
            sender_token: None,
            router_label: None,
            error_reason: None,
        };

        let mut packet_vec = vec![0; 2 + header.encoded_len() + request.encoded_len()];
        let mut packet = packet_vec.as_mut_slice();
        packet
            .write_u16::<BigEndian>(header.encoded_len() as u16)
            .unwrap();
        header.encode(&mut packet).unwrap();
        request.encode(&mut packet).unwrap();
        self.request_tx.send(packet_vec).unwrap();

        return header.token;
    }

    pub async fn request<M: Message + Default>(
        &self,
        service_hash: u32,
        method_id: u32,
        request: impl Message,
    ) -> M {
        let token = self.request_no_response(service_hash, method_id, request);

        let (response_tx, response_rx) = oneshot::channel();
        {
            let responders = self.responders.clone();
            let mut responders = responders.lock().unwrap();
            responders.insert(token, response_tx);
        }

        M::decode(response_rx.await.unwrap().as_slice()).unwrap()
    }

    #[rustfmt::skip]
    pub async fn handle_request(&self, service_hash: u32, method_id: u32, request: &[u8]) -> Option<Vec<u8>> {
        match service_hash {
            0x54DFDA17 => {
                use crate::bgs::protocol::account::v1::AccountListener;

                match method_id {
                    1 => {
                        self.on_account_state_updated(Message::decode(request).unwrap()).await;
                        None
                    }
                    2 => {
                        self.on_game_account_state_updated(Message::decode(request).unwrap()).await;
                        None
                    }
                    3 => {
                        self.on_game_accounts_updated(Message::decode(request).unwrap()).await;
                        None
                    }
                    4 => {
                        self.on_game_session_updated(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0x62DA0891 => {
                use crate::bgs::protocol::account::v1::AccountService;

                match method_id {
                    13 => Some(self.resolve_account(Message::decode(request).unwrap()).await.encode_to_vec()),
                    25 => Some(self.subscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    26 => Some(self.unsubscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    30 => Some(self.get_account_state(Message::decode(request).unwrap()).await.encode_to_vec()),
                    31 => Some(self.get_game_account_state(Message::decode(request).unwrap()).await.encode_to_vec()),
                    32 => Some(self.get_licenses(Message::decode(request).unwrap()).await.encode_to_vec()),
                    33 => Some(self.get_game_time_remaining_info(Message::decode(request).unwrap()).await.encode_to_vec()),
                    34 => Some(self.get_game_session_info(Message::decode(request).unwrap()).await.encode_to_vec()),
                    35 => Some(self.get_cais_info(Message::decode(request).unwrap()).await.encode_to_vec()),
                    37 => Some(self.get_authorized_data(Message::decode(request).unwrap()).await.encode_to_vec()),
                    44 => Some(self.get_signed_account_state(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x71240E35 => {
                use crate::bgs::protocol::authentication::v1::AuthenticationListener;

                match method_id {
                    4 => {
                        self.on_server_state_change(Message::decode(request).unwrap()).await;
                        None
                    }
                    5 => {
                        self.on_logon_complete(Message::decode(request).unwrap()).await;
                        None
                    }
                    10 => {
                        self.on_logon_update(Message::decode(request).unwrap()).await;
                        None
                    }
                    11 => {
                        self.on_version_info_updated(Message::decode(request).unwrap()).await;
                        None
                    }
                    12 => {
                        self.on_logon_queue_update(Message::decode(request).unwrap()).await;
                        None
                    }
                    13 => {
                        self.on_logon_queue_end(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0xDECFC01 => {
                use crate::bgs::protocol::authentication::v1::AuthenticationService;

                match method_id {
                    1 => Some(self.logon(Message::decode(request).unwrap()).await.encode_to_vec()),
                    7 => Some(self.verify_web_credentials(Message::decode(request).unwrap()).await.encode_to_vec()),
                    8 => Some(self.generate_web_credentials(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0xBBDA171F => {
                use crate::bgs::protocol::challenge::v1::ChallengeListener;

                match method_id {
                    3 => {
                        self.on_external_challenge(Message::decode(request).unwrap()).await;
                        None
                    }
                    4 => {
                        self.on_external_challenge_result(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0xBF8C8094 => {
                use crate::bgs::protocol::channel::v1::ChannelListener;

                match method_id {
                    1 => {
                        self.on_join(Message::decode(request).unwrap()).await;
                        None
                    }
                    2 => {
                        self.on_member_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    3 => {
                        self.on_leave(Message::decode(request).unwrap()).await;
                        None
                    }
                    4 => {
                        self.on_member_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    5 => {
                        self.on_send_message(Message::decode(request).unwrap()).await;
                        None
                    }
                    6 => {
                        self.on_update_channel_state(Message::decode(request).unwrap()).await;
                        None
                    }
                    7 => {
                        self.on_update_member_state(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0xB732DB32 => {
                use crate::bgs::protocol::channel::v1::ChannelService;

                match method_id {
                    2 => Some(self.remove_member(Message::decode(request).unwrap()).await.encode_to_vec()),
                    3 => Some(self.send_message(Message::decode(request).unwrap()).await.encode_to_vec()),
                    4 => Some(self.update_channel_state(Message::decode(request).unwrap()).await.encode_to_vec()),
                    5 => Some(self.update_member_state(Message::decode(request).unwrap()).await.encode_to_vec()),
                    6 => Some(self.dissolve(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x9890CDFE => {
                use crate::bgs::protocol::channel::v1::ChannelVoiceService;

                match method_id {
                    1 => Some(self.get_login_token(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.get_join_token(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x1AE52686 => {
                use crate::bgs::protocol::channel::v2::ChannelListener;

                match method_id {
                    3 => {
                        self.on_member_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    4 => {
                        self.on_member_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    5 => {
                        self.on_member_attribute_changed(Message::decode(request).unwrap()).await;
                        None
                    }
                    6 => {
                        self.on_member_role_changed(Message::decode(request).unwrap()).await;
                        None
                    }
                    10 => {
                        self.on_send_message(Message::decode(request).unwrap()).await;
                        None
                    }
                    11 => {
                        self.on_typing_indicator(Message::decode(request).unwrap()).await;
                        None
                    }
                    16 => {
                        self.on_attribute_changed(Message::decode(request).unwrap()).await;
                        None
                    }
                    17 => {
                        self.on_privacy_level_changed(Message::decode(request).unwrap()).await;
                        None
                    }
                    18 => {
                        self.on_invitation_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    19 => {
                        self.on_invitation_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    20 => {
                        self.on_suggestion_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0x798D39D1 => {
                use crate::bgs::protocol::channel::v2::ChannelService;

                match method_id {
                    2 => Some(self.create_channel(Message::decode(request).unwrap()).await.encode_to_vec()),
                    3 => Some(self.dissolve_channel(Message::decode(request).unwrap()).await.encode_to_vec()),
                    4 => Some(self.get_channel(Message::decode(request).unwrap()).await.encode_to_vec()),
                    5 => Some(self.get_public_channel_types(Message::decode(request).unwrap()).await.encode_to_vec()),
                    6 => Some(self.find_channel(Message::decode(request).unwrap()).await.encode_to_vec()),
                    10 => Some(self.subscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    11 => Some(self.unsubscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    21 => Some(self.set_attribute(Message::decode(request).unwrap()).await.encode_to_vec()),
                    22 => Some(self.set_privacy_level(Message::decode(request).unwrap()).await.encode_to_vec()),
                    23 => Some(self.send_message(Message::decode(request).unwrap()).await.encode_to_vec()),
                    24 => Some(self.set_typing_indicator(Message::decode(request).unwrap()).await.encode_to_vec()),
                    30 => Some(self.join(Message::decode(request).unwrap()).await.encode_to_vec()),
                    31 => Some(self.leave(Message::decode(request).unwrap()).await.encode_to_vec()),
                    32 => Some(self.kick(Message::decode(request).unwrap()).await.encode_to_vec()),
                    40 => Some(self.set_member_attribute(Message::decode(request).unwrap()).await.encode_to_vec()),
                    41 => Some(self.assign_role(Message::decode(request).unwrap()).await.encode_to_vec()),
                    42 => Some(self.unassign_role(Message::decode(request).unwrap()).await.encode_to_vec()),
                    50 => Some(self.send_invitation(Message::decode(request).unwrap()).await.encode_to_vec()),
                    51 => Some(self.accept_invitation(Message::decode(request).unwrap()).await.encode_to_vec()),
                    52 => Some(self.decline_invitation(Message::decode(request).unwrap()).await.encode_to_vec()),
                    53 => Some(self.revoke_invitation(Message::decode(request).unwrap()).await.encode_to_vec()),
                    60 => Some(self.send_suggestion(Message::decode(request).unwrap()).await.encode_to_vec()),
                    70 => Some(self.get_join_voice_token(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x18007BE => {
                use crate::bgs::protocol::channel::v2::membership::ChannelMembershipListener;

                match method_id {
                    1 => {
                        self.on_channel_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    2 => {
                        self.on_channel_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    3 => {
                        self.on_received_invitation_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    4 => {
                        self.on_received_invitation_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0x7E525E99 => {
                use crate::bgs::protocol::channel::v2::membership::ChannelMembershipService;

                match method_id {
                    1 => Some(self.subscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.unsubscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    3 => Some(self.get_state(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x2B34597B => {
                use crate::bgs::protocol::club::v1::membership::ClubMembershipListener;

                match method_id {
                    1 => {
                        self.on_club_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    2 => {
                        self.on_club_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    3 => {
                        self.on_received_invitation_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    4 => {
                        self.on_received_invitation_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    5 => {
                        self.on_shared_settings_changed(Message::decode(request).unwrap()).await;
                        None
                    }
                    6 => {
                        self.on_stream_mention_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    7 => {
                        self.on_stream_mention_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    8 => {
                        self.on_stream_mention_advance_view_time(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0x94B94786 => {
                use crate::bgs::protocol::club::v1::membership::ClubMembershipService;

                match method_id {
                    1 => Some(self.subscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.unsubscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    3 => Some(self.get_state(Message::decode(request).unwrap()).await.encode_to_vec()),
                    4 => Some(self.update_club_shared_settings(Message::decode(request).unwrap()).await.encode_to_vec()),
                    5 => Some(self.get_stream_mentions(Message::decode(request).unwrap()).await.encode_to_vec()),
                    6 => Some(self.remove_stream_mentions(Message::decode(request).unwrap()).await.encode_to_vec()),
                    7 => Some(self.advance_stream_mention_view_time(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x65446991 => {
                use crate::bgs::protocol::connection::v1::ConnectionService;

                match method_id {
                    1 => Some(self.connect(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.bind(Message::decode(request).unwrap()).await.encode_to_vec()),
                    3 => Some(self.echo(Message::decode(request).unwrap()).await.encode_to_vec()),
                    4 => {
                        self.force_disconnect(Message::decode(request).unwrap()).await;
                        None
                    }
                    5 => {
                        self.keep_alive(Message::decode(request).unwrap()).await;
                        None
                    }
                    6 => Some(self.encrypt(Message::decode(request).unwrap()).await.encode_to_vec()),
                    7 => {
                        self.request_disconnect(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0xB96F5297 => {
                use crate::bgs::protocol::diag::v1::DiagService;

                match method_id {
                    1 => Some(self.get_var(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.set_var(Message::decode(request).unwrap()).await.encode_to_vec()),
                    3 => Some(self.query(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x6F259A13 => {
                use crate::bgs::protocol::friends::v1::FriendsListener;

                match method_id {
                    1 => {
                        self.on_friend_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    2 => {
                        self.on_friend_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    3 => {
                        self.on_received_invitation_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    4 => {
                        self.on_received_invitation_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    5 => {
                        self.on_sent_invitation_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    6 => {
                        self.on_sent_invitation_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    7 => {
                        self.on_update_friend_state(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0xA3DDB1BD => {
                use crate::bgs::protocol::friends::v1::FriendsService;

                match method_id {
                    1 => Some(self.subscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.send_invitation(Message::decode(request).unwrap()).await.encode_to_vec()),
                    3 => Some(self.accept_invitation(Message::decode(request).unwrap()).await.encode_to_vec()),
                    4 => Some(self.revoke_invitation(Message::decode(request).unwrap()).await.encode_to_vec()),
                    5 => Some(self.decline_invitation(Message::decode(request).unwrap()).await.encode_to_vec()),
                    6 => Some(self.ignore_invitation(Message::decode(request).unwrap()).await.encode_to_vec()),
                    8 => Some(self.remove_friend(Message::decode(request).unwrap()).await.encode_to_vec()),
                    9 => Some(self.view_friends(Message::decode(request).unwrap()).await.encode_to_vec()),
                    10 => Some(self.update_friend_state(Message::decode(request).unwrap()).await.encode_to_vec()),
                    11 => Some(self.unsubscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    12 => Some(self.revoke_all_invitations(Message::decode(request).unwrap()).await.encode_to_vec()),
                    13 => Some(self.get_friend_list(Message::decode(request).unwrap()).await.encode_to_vec()),
                    14 => Some(self.create_friendship(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x5DBB51C2 => {
                use crate::bgs::protocol::game_utilities::v2::client::GameUtilitiesService;

                match method_id {
                    1 => Some(self.process_task(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.get_all_values_for_attribute(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x890AB85F => {
                use crate::bgs::protocol::presence::v1::PresenceListener;

                match method_id {
                    1 => {
                        self.on_subscribe(Message::decode(request).unwrap()).await;
                        None
                    }
                    2 => {
                        self.on_state_changed(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0xFA0796FF => {
                use crate::bgs::protocol::presence::v1::PresenceService;

                match method_id {
                    1 => Some(self.subscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.unsubscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    3 => Some(self.update(Message::decode(request).unwrap()).await.encode_to_vec()),
                    4 => Some(self.query(Message::decode(request).unwrap()).await.encode_to_vec()),
                    8 => Some(self.batch_subscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    9 => Some(self.batch_unsubscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x7CAF61C9 => {
                use crate::bgs::protocol::report::v1::ReportService;

                match method_id {
                    1 => Some(self.send_report(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.submit_report(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x3A4218FB => {
                use crate::bgs::protocol::report::v2::ReportService;

                match method_id {
                    1 => Some(self.submit_report(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0xECBE75BA => {
                use crate::bgs::protocol::resources::v1::ResourcesService;

                match method_id {
                    1 => Some(self.get_content_handle(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x7FE36B32 => {
                use crate::bgs::protocol::session::v1::SessionListener;

                match method_id {
                    1 => {
                        self.on_session_created(Message::decode(request).unwrap()).await;
                        None
                    }
                    2 => {
                        self.on_session_destroyed(Message::decode(request).unwrap()).await;
                        None
                    }
                    3 => {
                        self.on_session_updated(Message::decode(request).unwrap()).await;
                        None
                    }
                    4 => {
                        self.on_session_game_time_warning(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0x1E688C05 => {
                use crate::bgs::protocol::session::v1::SessionService;

                match method_id {
                    1 => Some(self.create_session(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.destroy_session(Message::decode(request).unwrap()).await.encode_to_vec()),
                    5 => Some(self.update_session(Message::decode(request).unwrap()).await.encode_to_vec()),
                    7 => Some(self.get_session_state_by_benefactor(Message::decode(request).unwrap()).await.encode_to_vec()),
                    8 => Some(self.mark_sessions_alive(Message::decode(request).unwrap()).await.encode_to_vec()),
                    9 => Some(self.get_session_state(Message::decode(request).unwrap()).await.encode_to_vec()),
                    10 => Some(self.get_signed_session_state(Message::decode(request).unwrap()).await.encode_to_vec()),
                    11 => Some(self.refresh_session_key(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0xD0FFDAEB => {
                use crate::bgs::protocol::sns::v1::SocialNetworkListener;

                match method_id {
                    1 => Some(self.on_facebook_bnet_friend_list_received(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x71DC8296 => {
                use crate::bgs::protocol::sns::v1::SocialNetworkService;

                match method_id {
                    1 => Some(self.get_facebook_auth_code(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.get_facebook_bnet_friends(Message::decode(request).unwrap()).await.encode_to_vec()),
                    3 => Some(self.get_facebook_settings(Message::decode(request).unwrap()).await.encode_to_vec()),
                    4 => Some(self.get_facebook_account_link_status(Message::decode(request).unwrap()).await.encode_to_vec()),
                    5 => Some(self.get_google_auth_token(Message::decode(request).unwrap()).await.encode_to_vec()),
                    6 => Some(self.get_google_settings(Message::decode(request).unwrap()).await.encode_to_vec()),
                    7 => Some(self.get_google_account_link_status(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0xBC872C22 => {
                use crate::bgs::protocol::user_manager::v1::UserManagerListener;

                match method_id {
                    1 => {
                        self.on_blocked_player_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    2 => {
                        self.on_blocked_player_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    11 => {
                        self.on_recent_players_added(Message::decode(request).unwrap()).await;
                        None
                    }
                    12 => {
                        self.on_recent_players_removed(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0x3E19268A => {
                use crate::bgs::protocol::user_manager::v1::UserManagerService;

                match method_id {
                    1 => Some(self.subscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    10 => Some(self.add_recent_players(Message::decode(request).unwrap()).await.encode_to_vec()),
                    11 => Some(self.clear_recent_players(Message::decode(request).unwrap()).await.encode_to_vec()),
                    20 => Some(self.block_player(Message::decode(request).unwrap()).await.encode_to_vec()),
                    21 => Some(self.unblock_player(Message::decode(request).unwrap()).await.encode_to_vec()),
                    40 => Some(self.block_player_for_session(Message::decode(request).unwrap()).await.encode_to_vec()),
                    51 => Some(self.unsubscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0xF5709E48 => {
                use crate::bgs::protocol::voice::v2::client::VoiceService;

                match method_id {
                    1 => Some(self.create_login_credentials(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.create_channel_stt_token(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            0x3FE5849E => {
                use crate::bgs::protocol::whisper::v1::WhisperListener;

                match method_id {
                    1 => {
                        self.on_whisper(Message::decode(request).unwrap()).await;
                        None
                    }
                    2 => {
                        self.on_whisper_echo(Message::decode(request).unwrap()).await;
                        None
                    }
                    3 => {
                        self.on_typing_indicator_update(Message::decode(request).unwrap()).await;
                        None
                    }
                    4 => {
                        self.on_advance_view_time(Message::decode(request).unwrap()).await;
                        None
                    }
                    5 => {
                        self.on_whisper_updated(Message::decode(request).unwrap()).await;
                        None
                    }
                    6 => {
                        self.on_advance_clear_time(Message::decode(request).unwrap()).await;
                        None
                    }
                    _ => None
                }
            }
            0xC12828F9 => {
                use crate::bgs::protocol::whisper::v1::WhisperService;

                match method_id {
                    1 => Some(self.subscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    2 => Some(self.unsubscribe(Message::decode(request).unwrap()).await.encode_to_vec()),
                    3 => Some(self.send_whisper(Message::decode(request).unwrap()).await.encode_to_vec()),
                    4 => Some(self.set_typing_indicator(Message::decode(request).unwrap()).await.encode_to_vec()),
                    5 => Some(self.advance_view_time(Message::decode(request).unwrap()).await.encode_to_vec()),
                    6 => Some(self.get_whisper_messages(Message::decode(request).unwrap()).await.encode_to_vec()),
                    7 => Some(self.advance_clear_time(Message::decode(request).unwrap()).await.encode_to_vec()),
                    _ => None
                }
            }
            _ => None
        }
    }

    pub fn handle_response(&self, token: u32, response: Vec<u8>) {
        let responders = self.responders.clone();
        let mut responders = responders.lock().unwrap();
        if let Some(response_tx) = responders.remove(&token) {
            response_tx.send(response).unwrap();
        }
    }
}

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
