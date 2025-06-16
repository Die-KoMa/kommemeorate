// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

diesel::table! {
    memes (id) {
        id -> Int4,
        spoiler -> Bool,
        text -> Text,
        timestamp -> Timestamp,
        account -> Text,
        channel -> Text,
        telegram_id -> Nullable<Int4>,
        filename -> Text,
    }
}
