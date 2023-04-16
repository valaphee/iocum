use std::collections::HashMap;

use byteorder::{BigEndian, ReadBytesExt};
use prost::Message;

pub fn print_bgs(
    message: tokio_tungstenite::tungstenite::Message,
    pending_responses: &mut HashMap<u32, (u32, u32)>,
    from_server: bool,
) {
    if let tokio_tungstenite::tungstenite::Message::Binary(data) = message {
        let mut data = data.as_slice();
        let header_size = data.read_u16::<BigEndian>().unwrap();
        let (header_data, data) = data.split_at(header_size as usize);
        let header = protocol::Header::decode(header_data).unwrap();
        match header.service_id {
            0 => {
                pending_responses.insert(
                    header.token,
                    (header.service_hash.unwrap(), header.method_id.unwrap()),
                );

                let client_or_server = if from_server { "CLIENT" } else { "SERVER" };
                println!(
                    "-----BEGIN {} RPC REQUEST #{}-----",
                    client_or_server, header.token
                );
                print_bgs_request(
                    header.service_hash.unwrap(),
                    header.method_id.unwrap(),
                    data,
                );
                println!("-----END {} RPC REQUEST-----", client_or_server);
            }
            254 => {
                if let Some((service_hash, method_id)) = pending_responses.remove(&header.token) {
                    let client_or_server = if from_server { "SERVER" } else { "CLIENT" };
                    println!(
                        "-----BEGIN {} RPC RESPONSE #{}-----",
                        client_or_server, header.token
                    );
                    print_bgs_response(service_hash, method_id, data);
                    println!("-----END {} RPC RESPONSE-----", client_or_server);
                }
            }
            _ => {}
        }
    }
}

fn print_bgs_request(service_hash: u32, method_id: u32, data: &[u8]) {
    let request: Box<dyn prost::Message> = match service_hash {
        0x54DFDA17 => match method_id {
            1 => Box::new(protocol::account::v1::AccountStateNotification::decode(data).unwrap()),
            2 => {
                Box::new(protocol::account::v1::GameAccountStateNotification::decode(data).unwrap())
            }
            3 => Box::new(protocol::account::v1::GameAccountNotification::decode(data).unwrap()),
            4 => Box::new(
                protocol::account::v1::GameAccountSessionNotification::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x62DA0891 => match method_id {
            13 => Box::new(protocol::account::v1::ResolveAccountRequest::decode(data).unwrap()),
            25 => Box::new(protocol::account::v1::SubscriptionUpdateRequest::decode(data).unwrap()),
            26 => Box::new(protocol::account::v1::SubscriptionUpdateRequest::decode(data).unwrap()),
            30 => Box::new(protocol::account::v1::GetAccountStateRequest::decode(data).unwrap()),
            31 => {
                Box::new(protocol::account::v1::GetGameAccountStateRequest::decode(data).unwrap())
            }
            32 => Box::new(protocol::account::v1::GetLicensesRequest::decode(data).unwrap()),
            33 => Box::new(
                protocol::account::v1::GetGameTimeRemainingInfoRequest::decode(data).unwrap(),
            ),
            34 => Box::new(protocol::account::v1::GetGameSessionInfoRequest::decode(data).unwrap()),
            37 => Box::new(protocol::account::v1::GetAuthorizedDataRequest::decode(data).unwrap()),
            44 => {
                Box::new(protocol::account::v1::GetSignedAccountStateRequest::decode(data).unwrap())
            }
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x71240E35 => match method_id {
            4 => Box::new(
                protocol::authentication::v1::ServerStateChangeRequest::decode(data).unwrap(),
            ),
            5 => Box::new(protocol::authentication::v1::LogonResult::decode(data).unwrap()),
            10 => Box::new(protocol::authentication::v1::LogonUpdateRequest::decode(data).unwrap()),
            11 => Box::new(
                protocol::authentication::v1::VersionInfoNotification::decode(data).unwrap(),
            ),
            12 => Box::new(
                protocol::authentication::v1::LogonQueueUpdateRequest::decode(data).unwrap(),
            ),
            13 => Box::new(protocol::NoData::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0xDECFC01 => match method_id {
            1 => Box::new(protocol::authentication::v1::LogonRequest::decode(data).unwrap()),
            7 => Box::new(
                protocol::authentication::v1::VerifyWebCredentialsRequest::decode(data).unwrap(),
            ),
            8 => Box::new(
                protocol::authentication::v1::GenerateWebCredentialsRequest::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0xBBDA171F => match method_id {
            3 => Box::new(protocol::challenge::v1::ChallengeExternalRequest::decode(data).unwrap()),
            4 => Box::new(protocol::challenge::v1::ChallengeExternalResult::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0xBF8C8094 => match method_id {
            1 => Box::new(protocol::channel::v1::JoinNotification::decode(data).unwrap()),
            2 => Box::new(protocol::channel::v1::MemberAddedNotification::decode(data).unwrap()),
            3 => Box::new(protocol::channel::v1::LeaveNotification::decode(data).unwrap()),
            4 => Box::new(protocol::channel::v1::MemberRemovedNotification::decode(data).unwrap()),
            5 => Box::new(protocol::channel::v1::SendMessageNotification::decode(data).unwrap()),
            6 => Box::new(
                protocol::channel::v1::UpdateChannelStateNotification::decode(data).unwrap(),
            ),
            7 => Box::new(
                protocol::channel::v1::UpdateMemberStateNotification::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0xB732DB32 => match method_id {
            2 => Box::new(protocol::channel::v1::RemoveMemberRequest::decode(data).unwrap()),
            3 => Box::new(protocol::channel::v1::SendMessageRequest::decode(data).unwrap()),
            4 => Box::new(protocol::channel::v1::UpdateChannelStateRequest::decode(data).unwrap()),
            5 => Box::new(protocol::channel::v1::UpdateMemberStateRequest::decode(data).unwrap()),
            6 => Box::new(protocol::channel::v1::DissolveRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x9890CDFE => match method_id {
            1 => Box::new(protocol::channel::v1::GetLoginTokenRequest::decode(data).unwrap()),
            2 => Box::new(protocol::channel::v1::GetJoinTokenRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x1AE52686 => match method_id {
            3 => Box::new(protocol::channel::v2::MemberAddedNotification::decode(data).unwrap()),
            4 => Box::new(protocol::channel::v2::MemberRemovedNotification::decode(data).unwrap()),
            5 => Box::new(
                protocol::channel::v2::MemberAttributeChangedNotification::decode(data).unwrap(),
            ),
            6 => Box::new(
                protocol::channel::v2::MemberRoleChangedNotification::decode(data).unwrap(),
            ),
            10 => Box::new(protocol::channel::v2::SendMessageNotification::decode(data).unwrap()),
            11 => {
                Box::new(protocol::channel::v2::TypingIndicatorNotification::decode(data).unwrap())
            }
            16 => {
                Box::new(protocol::channel::v2::AttributeChangedNotification::decode(data).unwrap())
            }
            17 => Box::new(
                protocol::channel::v2::PrivacyLevelChangedNotification::decode(data).unwrap(),
            ),
            18 => {
                Box::new(protocol::channel::v2::InvitationAddedNotification::decode(data).unwrap())
            }
            19 => Box::new(
                protocol::channel::v2::InvitationRemovedNotification::decode(data).unwrap(),
            ),
            20 => {
                Box::new(protocol::channel::v2::SuggestionAddedNotification::decode(data).unwrap())
            }
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x798D39D1 => match method_id {
            2 => Box::new(protocol::channel::v2::CreateChannelRequest::decode(data).unwrap()),
            3 => Box::new(protocol::channel::v2::DissolveChannelRequest::decode(data).unwrap()),
            4 => Box::new(protocol::channel::v2::GetChannelRequest::decode(data).unwrap()),
            5 => {
                Box::new(protocol::channel::v2::GetPublicChannelTypesRequest::decode(data).unwrap())
            }
            6 => Box::new(protocol::channel::v2::FindChannelRequest::decode(data).unwrap()),
            10 => Box::new(protocol::channel::v2::SubscribeRequest::decode(data).unwrap()),
            11 => Box::new(protocol::channel::v2::UnsubscribeRequest::decode(data).unwrap()),
            21 => Box::new(protocol::channel::v2::SetAttributeRequest::decode(data).unwrap()),
            22 => Box::new(protocol::channel::v2::SetPrivacyLevelRequest::decode(data).unwrap()),
            23 => Box::new(protocol::channel::v2::SendMessageRequest::decode(data).unwrap()),
            24 => Box::new(protocol::channel::v2::SetTypingIndicatorRequest::decode(data).unwrap()),
            30 => Box::new(protocol::channel::v2::JoinRequest::decode(data).unwrap()),
            31 => Box::new(protocol::channel::v2::LeaveRequest::decode(data).unwrap()),
            32 => Box::new(protocol::channel::v2::KickRequest::decode(data).unwrap()),
            40 => Box::new(protocol::channel::v2::SetMemberAttributeRequest::decode(data).unwrap()),
            41 => Box::new(protocol::channel::v2::AssignRoleRequest::decode(data).unwrap()),
            42 => Box::new(protocol::channel::v2::UnassignRoleRequest::decode(data).unwrap()),
            50 => Box::new(protocol::channel::v2::SendInvitationRequest::decode(data).unwrap()),
            51 => Box::new(protocol::channel::v2::AcceptInvitationRequest::decode(data).unwrap()),
            52 => Box::new(protocol::channel::v2::DeclineInvitationRequest::decode(data).unwrap()),
            53 => Box::new(protocol::channel::v2::RevokeInvitationRequest::decode(data).unwrap()),
            60 => Box::new(protocol::channel::v2::SendSuggestionRequest::decode(data).unwrap()),
            70 => Box::new(protocol::channel::v2::GetJoinVoiceTokenRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x18007BE => match method_id {
            1 => Box::new(
                protocol::channel::v2::membership::ChannelAddedNotification::decode(data).unwrap(),
            ),
            2 => Box::new(
                protocol::channel::v2::membership::ChannelRemovedNotification::decode(data)
                    .unwrap(),
            ),
            3 => Box::new(
                protocol::channel::v2::membership::ReceivedInvitationAddedNotification::decode(
                    data,
                )
                .unwrap(),
            ),
            4 => Box::new(
                protocol::channel::v2::membership::ReceivedInvitationRemovedNotification::decode(
                    data,
                )
                .unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x7E525E99 => match method_id {
            1 => {
                Box::new(protocol::channel::v2::membership::SubscribeRequest::decode(data).unwrap())
            }
            2 => Box::new(
                protocol::channel::v2::membership::UnsubscribeRequest::decode(data).unwrap(),
            ),
            3 => {
                Box::new(protocol::channel::v2::membership::GetStateRequest::decode(data).unwrap())
            }
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x2B34597B => match method_id {
            1 => Box::new(
                protocol::club::v1::membership::ClubAddedNotification::decode(data).unwrap(),
            ),
            2 => Box::new(
                protocol::club::v1::membership::ClubRemovedNotification::decode(data).unwrap(),
            ),
            3 => Box::new(
                protocol::club::v1::membership::ReceivedInvitationAddedNotification::decode(data)
                    .unwrap(),
            ),
            4 => Box::new(
                protocol::club::v1::membership::ReceivedInvitationRemovedNotification::decode(data)
                    .unwrap(),
            ),
            5 => Box::new(
                protocol::club::v1::membership::SharedSettingsChangedNotification::decode(data)
                    .unwrap(),
            ),
            6 => Box::new(
                protocol::club::v1::membership::StreamMentionAddedNotification::decode(data)
                    .unwrap(),
            ),
            7 => Box::new(
                protocol::club::v1::membership::StreamMentionRemovedNotification::decode(data)
                    .unwrap(),
            ),
            8 => Box::new(
                protocol::club::v1::membership::StreamMentionAdvanceViewTimeNotification::decode(
                    data,
                )
                .unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x94B94786 => match method_id {
            1 => Box::new(protocol::club::v1::membership::SubscribeRequest::decode(data).unwrap()),
            2 => {
                Box::new(protocol::club::v1::membership::UnsubscribeRequest::decode(data).unwrap())
            }
            3 => Box::new(protocol::club::v1::membership::GetStateRequest::decode(data).unwrap()),
            4 => Box::new(
                protocol::club::v1::membership::UpdateClubSharedSettingsRequest::decode(data)
                    .unwrap(),
            ),
            5 => Box::new(
                protocol::club::v1::membership::GetStreamMentionsRequest::decode(data).unwrap(),
            ),
            6 => Box::new(
                protocol::club::v1::membership::RemoveStreamMentionsRequest::decode(data).unwrap(),
            ),
            7 => Box::new(
                protocol::club::v1::membership::AdvanceStreamMentionViewTimeRequest::decode(data)
                    .unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x65446991 => match method_id {
            1 => Box::new(protocol::connection::v1::ConnectRequest::decode(data).unwrap()),
            2 => Box::new(protocol::connection::v1::BindRequest::decode(data).unwrap()),
            3 => Box::new(protocol::connection::v1::EchoRequest::decode(data).unwrap()),
            4 => Box::new(protocol::connection::v1::DisconnectNotification::decode(data).unwrap()),
            5 => Box::new(protocol::NoData::decode(data).unwrap()),
            6 => Box::new(protocol::connection::v1::EncryptRequest::decode(data).unwrap()),
            7 => Box::new(protocol::connection::v1::DisconnectRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0xB96F5297 => match method_id {
            1 => Box::new(protocol::diag::v1::GetVarRequest::decode(data).unwrap()),
            2 => Box::new(protocol::diag::v1::SetVarRequest::decode(data).unwrap()),
            3 => Box::new(protocol::diag::v1::QueryRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x6F259A13 => match method_id {
            1 => Box::new(protocol::friends::v1::FriendNotification::decode(data).unwrap()),
            2 => Box::new(protocol::friends::v1::FriendNotification::decode(data).unwrap()),
            3 => Box::new(protocol::friends::v1::InvitationNotification::decode(data).unwrap()),
            4 => Box::new(protocol::friends::v1::InvitationNotification::decode(data).unwrap()),
            5 => Box::new(
                protocol::friends::v1::SentInvitationAddedNotification::decode(data).unwrap(),
            ),
            6 => Box::new(
                protocol::friends::v1::SentInvitationRemovedNotification::decode(data).unwrap(),
            ),
            7 => Box::new(
                protocol::friends::v1::UpdateFriendStateNotification::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0xA3DDB1BD => match method_id {
            1 => Box::new(protocol::friends::v1::SubscribeRequest::decode(data).unwrap()),
            2 => Box::new(protocol::friends::v1::SendInvitationRequest::decode(data).unwrap()),
            3 => Box::new(protocol::friends::v1::AcceptInvitationRequest::decode(data).unwrap()),
            4 => Box::new(protocol::friends::v1::RevokeInvitationRequest::decode(data).unwrap()),
            5 => Box::new(protocol::friends::v1::DeclineInvitationRequest::decode(data).unwrap()),
            6 => Box::new(protocol::friends::v1::IgnoreInvitationRequest::decode(data).unwrap()),
            8 => Box::new(protocol::friends::v1::RemoveFriendRequest::decode(data).unwrap()),
            9 => Box::new(protocol::friends::v1::ViewFriendsRequest::decode(data).unwrap()),
            10 => Box::new(protocol::friends::v1::UpdateFriendStateRequest::decode(data).unwrap()),
            11 => Box::new(protocol::friends::v1::UnsubscribeRequest::decode(data).unwrap()),
            12 => {
                Box::new(protocol::friends::v1::RevokeAllInvitationsRequest::decode(data).unwrap())
            }
            13 => Box::new(protocol::friends::v1::GetFriendListRequest::decode(data).unwrap()),
            14 => Box::new(protocol::friends::v1::CreateFriendshipRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x5DBB51C2 => match method_id {
            1 => Box::new(
                protocol::game_utilities::v2::client::ProcessTaskRequest::decode(data).unwrap(),
            ),
            2 => Box::new(
                protocol::game_utilities::v2::client::GetAllValuesForAttributeRequest::decode(data)
                    .unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x890AB85F => match method_id {
            1 => Box::new(protocol::presence::v1::SubscribeNotification::decode(data).unwrap()),
            2 => Box::new(protocol::presence::v1::StateChangedNotification::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0xFA0796FF => match method_id {
            1 => Box::new(protocol::presence::v1::SubscribeRequest::decode(data).unwrap()),
            2 => Box::new(protocol::presence::v1::UnsubscribeRequest::decode(data).unwrap()),
            3 => Box::new(protocol::presence::v1::UpdateRequest::decode(data).unwrap()),
            4 => Box::new(protocol::presence::v1::QueryRequest::decode(data).unwrap()),
            8 => Box::new(protocol::presence::v1::BatchSubscribeRequest::decode(data).unwrap()),
            9 => Box::new(protocol::presence::v1::BatchUnsubscribeRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x7CAF61C9 => match method_id {
            1 => Box::new(protocol::report::v1::SendReportRequest::decode(data).unwrap()),
            2 => Box::new(protocol::report::v1::SubmitReportRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x3A4218FB => match method_id {
            1 => Box::new(protocol::report::v2::SubmitReportRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0xECBE75BA => match method_id {
            1 => Box::new(protocol::resources::v1::ContentHandleRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x7FE36B32 => match method_id {
            1 => Box::new(protocol::session::v1::SessionCreatedNotification::decode(data).unwrap()),
            2 => {
                Box::new(protocol::session::v1::SessionDestroyedNotification::decode(data).unwrap())
            }
            3 => Box::new(protocol::session::v1::SessionUpdatedNotification::decode(data).unwrap()),
            4 => Box::new(
                protocol::session::v1::SessionGameTimeWarningNotification::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x1E688C05 => match method_id {
            1 => Box::new(protocol::session::v1::CreateSessionRequest::decode(data).unwrap()),
            2 => Box::new(protocol::session::v1::DestroySessionRequest::decode(data).unwrap()),
            5 => Box::new(protocol::session::v1::UpdateSessionRequest::decode(data).unwrap()),
            7 => Box::new(
                protocol::session::v1::GetSessionStateByBenefactorRequest::decode(data).unwrap(),
            ),
            8 => Box::new(protocol::session::v1::MarkSessionsAliveRequest::decode(data).unwrap()),
            9 => Box::new(protocol::session::v1::GetSessionStateRequest::decode(data).unwrap()),
            10 => {
                Box::new(protocol::session::v1::GetSignedSessionStateRequest::decode(data).unwrap())
            }
            11 => Box::new(protocol::session::v1::RefreshSessionKeyRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0xD0FFDAEB => match method_id {
            1 => Box::new(
                protocol::sns::v1::FacebookBnetFriendListNotification::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x71DC8296 => match method_id {
            1 => Box::new(protocol::sns::v1::GetFacebookAuthCodeRequest::decode(data).unwrap()),
            2 => Box::new(protocol::sns::v1::GetFacebookBnetFriendsRequest::decode(data).unwrap()),
            3 => Box::new(protocol::NoData::decode(data).unwrap()),
            4 => Box::new(
                protocol::sns::v1::GetFacebookAccountLinkStatusRequest::decode(data).unwrap(),
            ),
            5 => Box::new(protocol::sns::v1::GetGoogleAuthTokenRequest::decode(data).unwrap()),
            6 => Box::new(protocol::NoData::decode(data).unwrap()),
            7 => Box::new(
                protocol::sns::v1::GetGoogleAccountLinkStatusRequest::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0xBC872C22 => match method_id {
            1 => Box::new(
                protocol::user_manager::v1::BlockedPlayerAddedNotification::decode(data).unwrap(),
            ),
            2 => Box::new(
                protocol::user_manager::v1::BlockedPlayerRemovedNotification::decode(data).unwrap(),
            ),
            11 => Box::new(
                protocol::user_manager::v1::RecentPlayersAddedNotification::decode(data).unwrap(),
            ),
            12 => Box::new(
                protocol::user_manager::v1::RecentPlayersRemovedNotification::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x3E19268A => match method_id {
            1 => Box::new(protocol::user_manager::v1::SubscribeRequest::decode(data).unwrap()),
            10 => {
                Box::new(protocol::user_manager::v1::AddRecentPlayersRequest::decode(data).unwrap())
            }
            11 => Box::new(
                protocol::user_manager::v1::ClearRecentPlayersRequest::decode(data).unwrap(),
            ),
            20 => Box::new(protocol::user_manager::v1::BlockPlayerRequest::decode(data).unwrap()),
            21 => Box::new(protocol::user_manager::v1::UnblockPlayerRequest::decode(data).unwrap()),
            40 => Box::new(protocol::user_manager::v1::BlockPlayerRequest::decode(data).unwrap()),
            51 => Box::new(protocol::user_manager::v1::UnsubscribeRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0xF5709E48 => match method_id {
            1 => Box::new(
                protocol::voice::v2::client::CreateLoginCredentialsRequest::decode(data).unwrap(),
            ),
            2 => Box::new(
                protocol::voice::v2::client::CreateChannelSttTokenRequest::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0x3FE5849E => match method_id {
            1 => Box::new(protocol::whisper::v1::WhisperNotification::decode(data).unwrap()),
            2 => Box::new(protocol::whisper::v1::WhisperEchoNotification::decode(data).unwrap()),
            3 => {
                Box::new(protocol::whisper::v1::TypingIndicatorNotification::decode(data).unwrap())
            }
            4 => {
                Box::new(protocol::whisper::v1::AdvanceViewTimeNotification::decode(data).unwrap())
            }
            5 => Box::new(protocol::whisper::v1::WhisperUpdatedNotification::decode(data).unwrap()),
            6 => {
                Box::new(protocol::whisper::v1::AdvanceClearTimeNotification::decode(data).unwrap())
            }
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        0xC12828F9 => match method_id {
            1 => Box::new(protocol::whisper::v1::SubscribeRequest::decode(data).unwrap()),
            2 => Box::new(protocol::whisper::v1::UnsubscribeRequest::decode(data).unwrap()),
            3 => Box::new(protocol::whisper::v1::SendWhisperRequest::decode(data).unwrap()),
            4 => Box::new(protocol::whisper::v1::SetTypingIndicatorRequest::decode(data).unwrap()),
            5 => Box::new(protocol::whisper::v1::AdvanceViewTimeRequest::decode(data).unwrap()),
            6 => Box::new(protocol::whisper::v1::GetWhisperMessagesRequest::decode(data).unwrap()),
            7 => Box::new(protocol::whisper::v1::AdvanceClearTimeRequest::decode(data).unwrap()),
            _ => Box::new(protocol::NoData::decode(data).unwrap()),
        },
        _ => Box::new(protocol::NoData::decode(data).unwrap()),
    };
    println!("{:?}", request);
}

fn print_bgs_response(service_hash: u32, method_id: u32, data: &[u8]) {
    let response: Box<dyn prost::Message> = match service_hash {
        0x54DFDA17 => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x62DA0891 => match method_id {
            13 => Box::new(protocol::account::v1::ResolveAccountResponse::decode(data).unwrap()),
            25 => {
                Box::new(protocol::account::v1::SubscriptionUpdateResponse::decode(data).unwrap())
            }
            26 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            30 => Box::new(protocol::account::v1::GetAccountStateResponse::decode(data).unwrap()),
            31 => {
                Box::new(protocol::account::v1::GetGameAccountStateResponse::decode(data).unwrap())
            }
            32 => Box::new(protocol::account::v1::GetLicensesResponse::decode(data).unwrap()),
            33 => Box::new(
                protocol::account::v1::GetGameTimeRemainingInfoResponse::decode(data).unwrap(),
            ),
            34 => {
                Box::new(protocol::account::v1::GetGameSessionInfoResponse::decode(data).unwrap())
            }
            35 => Box::new(protocol::account::v1::GetCaisInfoResponse::decode(data).unwrap()),
            37 => Box::new(protocol::account::v1::GetAuthorizedDataResponse::decode(data).unwrap()),
            44 => Box::new(
                protocol::account::v1::GetSignedAccountStateResponse::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x71240E35 => match method_id {
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            5 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            10 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            11 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            12 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            13 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0xDECFC01 => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            7 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            8 => Box::new(
                protocol::authentication::v1::GenerateWebCredentialsResponse::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0xBBDA171F => match method_id {
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0xBF8C8094 => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            5 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            6 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            7 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0xB732DB32 => match method_id {
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            5 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            6 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x9890CDFE => match method_id {
            1 => Box::new(protocol::channel::v1::GetLoginTokenResponse::decode(data).unwrap()),
            2 => Box::new(protocol::channel::v1::GetJoinTokenResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x1AE52686 => match method_id {
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            5 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            6 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            10 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            11 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            16 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            17 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            18 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            19 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            20 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x798D39D1 => match method_id {
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::channel::v2::GetChannelResponse::decode(data).unwrap()),
            5 => Box::new(
                protocol::channel::v2::GetPublicChannelTypesResponse::decode(data).unwrap(),
            ),
            6 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            10 => Box::new(protocol::channel::v2::SubscribeResponse::decode(data).unwrap()),
            11 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            21 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            22 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            23 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            24 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            30 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            31 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            32 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            40 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            41 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            42 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            50 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            51 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            52 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            53 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            60 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            70 => Box::new(protocol::channel::v2::GetJoinVoiceTokenResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x18007BE => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x7E525E99 => match method_id {
            1 => Box::new(
                protocol::channel::v2::membership::SubscribeResponse::decode(data).unwrap(),
            ),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => {
                Box::new(protocol::channel::v2::membership::GetStateResponse::decode(data).unwrap())
            }
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x2B34597B => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            5 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            6 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            7 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            8 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x94B94786 => match method_id {
            1 => Box::new(protocol::club::v1::membership::SubscribeResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::club::v1::membership::GetStateResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            5 => Box::new(
                protocol::club::v1::membership::GetStreamMentionsResponse::decode(data).unwrap(),
            ),
            6 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            7 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x65446991 => match method_id {
            1 => Box::new(protocol::connection::v1::ConnectResponse::decode(data).unwrap()),
            2 => Box::new(protocol::connection::v1::BindResponse::decode(data).unwrap()),
            3 => Box::new(protocol::connection::v1::EchoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            5 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            6 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            7 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0xB96F5297 => match method_id {
            1 => Box::new(protocol::diag::v1::GetVarResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::diag::v1::QueryResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x6F259A13 => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            5 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            6 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            7 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0xA3DDB1BD => match method_id {
            1 => Box::new(protocol::friends::v1::SubscribeResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            5 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            6 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            8 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            9 => Box::new(protocol::friends::v1::ViewFriendsResponse::decode(data).unwrap()),
            10 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            11 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            12 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            13 => Box::new(protocol::friends::v1::GetFriendListResponse::decode(data).unwrap()),
            14 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x5DBB51C2 => match method_id {
            1 => Box::new(
                protocol::game_utilities::v2::client::ProcessTaskResponse::decode(data).unwrap(),
            ),
            2 => Box::new(
                protocol::game_utilities::v2::client::GetAllValuesForAttributeResponse::decode(
                    data,
                )
                .unwrap(),
            ),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x890AB85F => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0xFA0796FF => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::presence::v1::QueryResponse::decode(data).unwrap()),
            8 => Box::new(protocol::presence::v1::BatchSubscribeResponse::decode(data).unwrap()),
            9 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x7CAF61C9 => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x3A4218FB => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0xECBE75BA => match method_id {
            1 => Box::new(protocol::ContentHandle::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x7FE36B32 => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x1E688C05 => match method_id {
            1 => Box::new(protocol::session::v1::CreateSessionResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            5 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            7 => Box::new(
                protocol::session::v1::GetSessionStateByBenefactorResponse::decode(data).unwrap(),
            ),
            8 => Box::new(protocol::session::v1::MarkSessionsAliveResponse::decode(data).unwrap()),
            9 => Box::new(protocol::session::v1::GetSessionStateResponse::decode(data).unwrap()),
            10 => Box::new(
                protocol::session::v1::GetSignedSessionStateResponse::decode(data).unwrap(),
            ),
            11 => Box::new(protocol::session::v1::RefreshSessionKeyResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0xD0FFDAEB => match method_id {
            1 => Box::new(
                protocol::sns::v1::FacebookBnetFriendListNotificationResponse::decode(data)
                    .unwrap(),
            ),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x71DC8296 => match method_id {
            1 => Box::new(protocol::sns::v1::GetFacebookAuthCodeResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::sns::v1::GetFacebookSettingsResponse::decode(data).unwrap()),
            4 => Box::new(
                protocol::sns::v1::GetFacebookAccountLinkStatusResponse::decode(data).unwrap(),
            ),
            5 => Box::new(protocol::sns::v1::GetGoogleAuthTokenResponse::decode(data).unwrap()),
            6 => Box::new(protocol::sns::v1::GetGoogleSettingsResponse::decode(data).unwrap()),
            7 => Box::new(
                protocol::sns::v1::GetGoogleAccountLinkStatusResponse::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0xBC872C22 => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            11 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            12 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x3E19268A => match method_id {
            1 => Box::new(protocol::user_manager::v1::SubscribeResponse::decode(data).unwrap()),
            10 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            11 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            20 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            21 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            40 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            51 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0xF5709E48 => match method_id {
            1 => Box::new(
                protocol::voice::v2::client::CreateLoginCredentialsResponse::decode(data).unwrap(),
            ),
            2 => Box::new(
                protocol::voice::v2::client::CreateChannelSttTokenResponse::decode(data).unwrap(),
            ),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0x3FE5849E => match method_id {
            1 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            5 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            6 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        0xC12828F9 => match method_id {
            1 => Box::new(protocol::whisper::v1::SubscribeResponse::decode(data).unwrap()),
            2 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            3 => Box::new(protocol::whisper::v1::SendWhisperResponse::decode(data).unwrap()),
            4 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            5 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            6 => Box::new(protocol::whisper::v1::GetWhisperMessagesResponse::decode(data).unwrap()),
            7 => Box::new(protocol::NoResponse::decode(data).unwrap()),
            _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
        },
        _ => Box::new(protocol::NoResponse::decode(data).unwrap()),
    };
    println!("{:?}", response);
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
