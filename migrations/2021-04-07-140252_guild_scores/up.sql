-- Your SQL goes here
create table server_users (
    user_id bigint,
    server_id bigint,
    points bigint not null default 0,
    longest_chains int[3] not null default '{0,0,0}'::int[3],
    primary key (server_id, user_id)
);