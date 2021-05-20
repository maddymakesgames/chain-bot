-- Your SQL goes here
create table users (
    id bigint primary key,
    points bigint not null,
    longest_chains int[3] not null
)