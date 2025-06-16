// © 2025 Maximilian Marx
// SPDX-FileContributor: Maximilian Marx
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::NaiveDateTime;
use diesel::prelude::*;

#[allow(unused)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = super::schema::memes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct Meme {
    pub(crate) id: i32,
    pub(crate) spoiler: bool,
    pub(crate) text: String,
    pub(crate) timestamp: NaiveDateTime,
    pub(crate) account: String,
    pub(crate) channel: String,
    pub(crate) telegram_id: Option<i32>,
    pub(crate) filename: String,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::memes)]
pub(crate) struct NewMeme<'a> {
    pub(crate) spoiler: bool,
    pub(crate) text: &'a str,
    pub(crate) timestamp: NaiveDateTime,
    pub(crate) account: &'a str,
    pub(crate) channel: &'a str,
    pub(crate) telegram_id: Option<i32>,
    pub(crate) filename: &'a str,
}
