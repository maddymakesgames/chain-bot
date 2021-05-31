-- Your SQL goes here
create table guilds (
    id bigint primary key,
    prefixes text[] not null default '{"cb."}'::text[],
    channel_filters bigint[] not null default '{}'::bigint[],
    blacklist boolean not null default true,
    style text not null default 'embed',
    remove_messages boolean not null default true,
    chain_threshold smallint not null default 6,
    alternate_member boolean not null default true
);