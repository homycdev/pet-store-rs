create table if not exists pets(
    id BIGINT UNSIGNED primary key AUTO_INCREMENT not null,
    category varchar,
    photo_urls varchar,
    tags varchar,
    status varchar not null
);
create table if not exists "orders"(
    id BIGINT UNSIGNED primary key AUTO_INCREMENT not null,
    pet_id BIGINT,
    quantity varchar,
    ship_date datetime,
    status varchar not null,
    complete boolean,
    foreign key(pet_id) references pet(id)
);
create table if not exists "order_details" (
    id BIGINT UNSIGNED primary key AUTO_INCREMENT not null,
    order_id BIGINT not null,
    delivered datetime,
    details varchar,
    foreign key(order_id) references "orders"(id)
);