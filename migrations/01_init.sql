create table
    if not exists users (
        id bigserial primary key not null,
        username varchar not null,
        email varchar not null,
        password varchar not null
    );

create table
    if not exists pets (
        id bigserial primary key not null,
        category varchar,
        photo_urls varchar,
        tags varchar,
        status varchar not null
    );

create table
    if not exists "orders" (
        id bigserial primary key not null,
        pet_id BIGINT not null,
        user_id BIGINT not null,
        quantity BIGINT not null,
        ship_date timestamptz,
        status varchar not null
    );
