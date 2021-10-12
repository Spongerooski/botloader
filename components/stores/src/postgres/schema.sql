CREATE TABLE IF NOT EXISTS joined_guilds (
    id bigint PRIMARY KEY,
    name text NOT NULL,
    icon text NOT NULL,
    owner_id bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS guild_scripts (
    id bigserial PRIMARY KEY,
    guild_id bigint NOT NULL,
    name text NOT NULL,
    original_source text NOT NULL,
    compiled_js text NOT NULL,
    enabled boolean NOT NULL,
    UNIQUE (guild_id, name)
);

CREATE INDEX IF NOT EXISTS guild_scripts_guild_id_name_idx ON guild_scripts (guild_id, name);

CREATE TABLE IF NOT EXISTS script_links (
    id bigserial PRIMARY KEY,
    guild_id bigint NOT NULL,
    script_id bigint REFERENCES guild_scripts (id) ON DELETE CASCADE NOT NULL,
    context_type smallint NOT NULL,
    context_id bigint NOT NULL,
    UNIQUE (script_id, context_type, context_id)
);

CREATE INDEX IF NOT EXISTS script_links_guild_id_context_id_idx ON script_links (guild_id, context_type, context_id);

CREATE TABLE IF NOT EXISTS guild_meta_configs (
    guild_id bigserial PRIMARY KEY,
    error_channel_id bigint NOT NULL
);

