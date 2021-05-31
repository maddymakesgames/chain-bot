table! {
    guilds (id) {
        id -> Int8,
        prefixes -> Array<Text>,
        channel_filters -> Array<Int8>,
        blacklist -> Bool,
        style -> Text,
        remove_messages -> Bool,
        chain_threshold -> Int2,
        alternate_member -> Bool,
    }
}

table! {
    server_users (server_id, user_id) {
        user_id -> Int8,
        server_id -> Int8,
        points -> Int8,
        longest_chains -> Array<Int4>,
    }
}

table! {
    users (id) {
        id -> Int8,
        points -> Int8,
        longest_chains -> Array<Int4>,
    }
}

allow_tables_to_appear_in_same_query!(guilds, server_users, users,);
