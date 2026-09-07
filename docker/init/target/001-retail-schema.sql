create table if not exists public.sales (
    id text primary key,
    store_id text not null,
    customer_id text not null,
    amount_cents text not null,
    status text not null,
    updated_at text
);

create table if not exists public.sale_items (
    id text primary key,
    sale_id text not null,
    sku text not null,
    quantity text not null,
    updated_at text
);

create table if not exists public.payments (
    id text primary key,
    sale_id text not null,
    amount_cents text not null,
    method text not null,
    updated_at text
);
