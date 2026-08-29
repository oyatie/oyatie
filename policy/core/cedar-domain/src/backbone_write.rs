//! The backbone write-operation policy pack: one policy version per
//! implemented write action.

use crate::policy::{PolicyEffect, PolicyRuleInput, PolicyScope, PolicyVersion};

pub const BACKBONE_WRITE_POLICY_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackboneWriteOperation {
    MessengerPostMessage,
    MailSubmitMessage,
    SocialPublishPost,
    CommunityCreatePost,
    CommunityCastVote,
    CommunityApplyModerationAction,
}

impl BackboneWriteOperation {
    pub const fn all() -> [Self; 6] {
        [
            Self::MessengerPostMessage,
            Self::MailSubmitMessage,
            Self::SocialPublishPost,
            Self::CommunityCreatePost,
            Self::CommunityCastVote,
            Self::CommunityApplyModerationAction,
        ]
    }

    pub const fn policy_id(self) -> &'static str {
        match self {
            Self::MessengerPostMessage => "pol_backbone_messenger_message_post",
            Self::MailSubmitMessage => "pol_backbone_mail_message_submit",
            Self::SocialPublishPost => "pol_backbone_social_post_publish",
            Self::CommunityCreatePost => "pol_backbone_community_post_create",
            Self::CommunityCastVote => "pol_backbone_community_vote_cast",
            Self::CommunityApplyModerationAction => "pol_backbone_community_moderation_apply",
        }
    }

    pub const fn action(self) -> &'static str {
        match self {
            Self::MessengerPostMessage => "messenger.message.post",
            Self::MailSubmitMessage => "mail.message.submit",
            Self::SocialPublishPost => "social.post.publish",
            Self::CommunityCreatePost => "community.post.create",
            Self::CommunityCastVote => "community.vote.cast",
            Self::CommunityApplyModerationAction => "community.moderation.apply",
        }
    }

    pub const fn principal_role(self) -> &'static str {
        match self {
            Self::MessengerPostMessage => "messenger-writer",
            Self::MailSubmitMessage => "mail-sender",
            Self::SocialPublishPost => "social-publisher",
            Self::CommunityCreatePost => "community-author",
            Self::CommunityCastVote => "community-voter",
            Self::CommunityApplyModerationAction => "community-moderator",
        }
    }

    pub const fn resource_type(self) -> &'static str {
        match self {
            Self::MessengerPostMessage => "messenger:channel",
            Self::MailSubmitMessage => "mail:mailbox",
            Self::SocialPublishPost => "social:profile",
            Self::CommunityCreatePost => "community:space",
            Self::CommunityCastVote | Self::CommunityApplyModerationAction => "community:post",
        }
    }

    pub const fn resource_prefix(self) -> &'static str {
        match self {
            Self::MessengerPostMessage => "messenger:channel:",
            Self::MailSubmitMessage => "mail:mailbox:",
            Self::SocialPublishPost => "social:profile:",
            Self::CommunityCreatePost => "community:space:",
            Self::CommunityCastVote | Self::CommunityApplyModerationAction => "community:post:",
        }
    }
}

pub fn backbone_write_policy_versions(tenant_id: impl Into<String>) -> Vec<PolicyVersion> {
    let tenant_id = tenant_id.into();
    BackboneWriteOperation::all()
        .into_iter()
        .map(|operation| PolicyVersion {
            policy_id: operation.policy_id().to_string(),
            version: BACKBONE_WRITE_POLICY_VERSION.to_string(),
            scope: PolicyScope::Tenant(tenant_id.clone()),
            supersedes: None,
            rules: vec![PolicyRuleInput {
                effect: PolicyEffect::Allow,
                principal_role: operation.principal_role().to_string(),
                action: operation.action().to_string(),
                resource_prefix: operation.resource_prefix().to_string(),
                required_attribute: Some(("data_plane".to_string(), "backbone".to_string())),
                annotations: Vec::new(),
            }],
        })
        .collect()
}
