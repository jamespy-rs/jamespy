use crate::utils::misc::overwrite_to_string;
use poise::serenity_prelude::{Context, PermissionOverwriteType, Permissions};
use serenity::all::GuildId;

pub async fn get_permission_changes(
    ctx: Context,
    guild_id: GuildId,
    old_allow: Permissions,
    new_allow: Permissions,
    old_deny: Permissions,
    new_deny: Permissions,
    kind: PermissionOverwriteType,
) -> String {
    let name = match kind {
        PermissionOverwriteType::Member(user_id) => match user_id.to_user(ctx).await {
            Ok(user) => user.name.to_string(),
            Err(_) => String::from("Unknown User"),
        },
        PermissionOverwriteType::Role(role_id) => ctx
            .cache
            .guild(guild_id)
            .unwrap()
            .roles
            .get(&role_id)
            .map_or_else(|| "Unknown Role".to_string(), |role| role.name.to_string()),
        _ => String::from("Unknown"),
    };

    let mut changes_str = String::new();
    let kind_string = overwrite_to_string(kind);
    if old_allow != new_allow || old_deny != new_deny {
        changes_str.push_str(&format!(
            "Permission override for {name} ({kind_string}) changed!\n"
        ));
        let allow_changes_detail = get_permission_changes_detail(old_allow, new_allow, true);
        let deny_changes_detail = get_permission_changes_detail(old_deny, new_deny, false);

        if !allow_changes_detail.is_empty() {
            changes_str.push_str("allow:\n");
            changes_str.push_str(&allow_changes_detail);
        }

        if !deny_changes_detail.is_empty() {
            changes_str.push_str("deny:\n");
            changes_str.push_str(&deny_changes_detail);
        }
    }

    changes_str
}

pub fn get_permission_changes_detail(old: Permissions, new: Permissions, allow: bool) -> String {
    let mut changes_str = String::new();
    let added_color = if allow { "\x1B[92m" } else { "\x1B[31m" };
    let removed_color = if allow { "\x1B[31m" } else { "\x1B[92m" };

    let added_perms: Vec<String> = {
        let mut added = Vec::new();
        for permission in Permissions::all().iter() {
            let permission_name = format!("{permission}");
            if new.contains(permission) && !old.contains(permission) {
                added.push(permission_name);
            }
        }
        added
    };

    let removed_perms: Vec<String> = {
        let mut removed = Vec::new();
        for permission in Permissions::all().iter() {
            let permission_name = format!("{permission}");
            if !new.contains(permission) && old.contains(permission) {
                removed.push(permission_name);
            }
        }
        removed
    };

    if !added_perms.is_empty() {
        for perm in &added_perms {
            changes_str.push_str(&format!("{added_color}+ {perm}\n\x1B[0m"));
        }
    }

    if !removed_perms.is_empty() {
        for perm in &removed_perms {
            changes_str.push_str(&format!("{removed_color}- {perm}\n\x1B[0m"));
        }
    }

    changes_str
}
