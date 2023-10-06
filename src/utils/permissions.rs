use poise::serenity_prelude::{Context, PermissionOverwriteType, Permissions};

pub async fn get_permission_changes(
    ctx: Context,
    old_allow: Permissions,
    new_allow: Permissions,
    old_deny: Permissions,
    new_deny: Permissions,
    kind: PermissionOverwriteType,
) -> String {
    let kind_string = match kind {
        PermissionOverwriteType::Member(user_id) => match user_id.to_user(ctx).await {
            Ok(user) => user.name,
            Err(_) => String::from("Unknown User"),
        },
        PermissionOverwriteType::Role(role_id) => role_id
            .to_role_cached(ctx)
            .map(|role| role.name.to_string())
            .unwrap_or_else(|| "Unknown Role".to_string()),
        _ => String::from("Unknown"),
    };

    let mut changes_str = format!("Permission override for {} changed!\n", kind_string);
    changes_str.push_str("allow:\n");
    changes_str.push_str(&get_permission_changes_detail(old_allow, new_allow));
    changes_str.push_str("deny:\n");
    changes_str.push_str(&get_permission_changes_detail(old_deny, new_deny));
    changes_str
}

pub fn get_permission_changes_detail(old: Permissions, new: Permissions) -> String {
    let mut changes_str = String::new();
    let added_perms: Vec<String> = {
        let mut added = Vec::new();
        for permission in Permissions::all().iter() {
            let permission_name = format!("{}", permission);
            if new.contains(permission) && !old.contains(permission) {
                added.push(permission_name);
            }
        }
        added
    };

    let removed_perms: Vec<String> = {
        let mut removed = Vec::new();
        for permission in Permissions::all().iter() {
            let permission_name = format!("{}", permission);
            if !new.contains(permission) && old.contains(permission) {
                removed.push(permission_name);
            }
        }
        removed
    };

    if !added_perms.is_empty() {
        for perm in &added_perms {
            changes_str.push_str(&format!("\x1B[92m+ {}\n\x1B[0m", perm));
        }
    }

    if !removed_perms.is_empty() {
        for perm in &removed_perms {
            changes_str.push_str(&format!("\x1B[31m- {}\n\x1B[0m", perm));
        }
    }

    changes_str
}
