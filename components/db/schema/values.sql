create table if not exists kvstore_values (
    guild_id    bigint references joined_guilds(snowflake) on delete cascade not null,
    pack_id     bigint references packs(id)                on delete cascade not null,
    key_text    text not null,
    val         text not null,
    expire_at   timestamp with time zone,
    primary key(guild_id, pack_id, key_text)
);