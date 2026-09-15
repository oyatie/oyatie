use crate::Error;
use crate::authority::{
    AdmitCommand, ban_allowed, join_allowed, member_can_leave, member_can_send, valid_user,
};
use crate::record::AuthorityRecord;

impl AuthorityRecord {
    pub fn apply_message(
        &mut self,
        command: &AdmitCommand,
        origin_server_ts: u64,
    ) -> Result<(), Error> {
        let membership = self
            .rooms
            .get(&command.room)
            .and_then(|room| room.membership(&command.sender).map(str::to_owned));
        if !member_can_send(membership.as_deref().unwrap_or("")) {
            return Err(Error::Denied);
        }
        self.append(
            &command.room,
            &command.sender,
            &command.event_type,
            None,
            command.content.clone(),
            origin_server_ts,
        )?;
        Ok(())
    }

    pub fn apply_member(
        &mut self,
        command: &AdmitCommand,
        origin_server_ts: u64,
    ) -> Result<(), Error> {
        let target = command
            .state_key
            .as_deref()
            .filter(|id| valid_user(id))
            .ok_or_else(|| Error::Invalid("member events require a user state key".into()))?;
        let next_membership = command.content["membership"]
            .as_str()
            .ok_or_else(|| Error::Invalid("membership required".into()))?;
        let room = self
            .rooms
            .get(&command.room)
            .ok_or_else(|| Error::Invalid("unknown room".into()))?;
        let current = room.membership(target).map(str::to_owned);
        let actor_membership = room.membership(&command.sender).map(str::to_owned);
        match next_membership {
            "join" => {
                if command.sender != target {
                    return Err(Error::Denied);
                }
                join_allowed(current.as_deref(), &room.join_rule)?;
            }
            "leave" => {
                if command.sender == target {
                    if !member_can_leave(current.as_deref().unwrap_or("")) {
                        return Err(Error::Denied);
                    }
                } else if !member_can_send(actor_membership.as_deref().unwrap_or(""))
                    || ban_allowed(room.creator == command.sender, room.creator == target).is_err()
                {
                    return Err(Error::Denied);
                }
            }
            "invite" => {
                if !member_can_send(actor_membership.as_deref().unwrap_or(""))
                    || matches!(current.as_deref(), Some("ban" | "join"))
                {
                    return Err(Error::Denied);
                }
            }
            "ban" => {
                if !member_can_send(actor_membership.as_deref().unwrap_or("")) {
                    return Err(Error::Denied);
                }
                ban_allowed(room.creator == command.sender, room.creator == target)?;
            }
            _ => return Err(Error::Invalid("unsupported membership".into())),
        }
        self.append(
            &command.room,
            &command.sender,
            "m.room.member",
            Some(target.into()),
            command.content.clone(),
            origin_server_ts,
        )?;
        let room = self
            .rooms
            .get_mut(&command.room)
            .ok_or_else(|| Error::Unavailable("room missing during membership".into()))?;
        room.members.insert(target.into(), next_membership.into());
        Ok(())
    }
}
