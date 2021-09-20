create table joined_guilds (
    snowflake   bigint  primary key,
    owner_id    bigint  not null,
    guild_name  text    not null,
    vanity_url  text,
    unique(vanity_url)
);

create table packs (
    id          bigserial   primary key,
    pack_name   text
);