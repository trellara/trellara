create table if not exists public.sales (
    id text primary key,
    store_id text not null,
    customer_id text not null,
    amount_cents text not null,
    status text not null,
    updated_at timestamptz not null default now()
);

create table if not exists public.sale_items (
    id text primary key,
    sale_id text not null references public.sales(id),
    sku text not null,
    quantity text not null,
    updated_at timestamptz not null default now()
);

create table if not exists public.payments (
    id text primary key,
    sale_id text not null references public.sales(id),
    amount_cents text not null,
    method text not null,
    updated_at timestamptz not null default now()
);

alter table public.sales replica identity full;
alter table public.sale_items replica identity full;
alter table public.payments replica identity full;

insert into public.sales (id, store_id, customer_id, amount_cents, status)
values ('sale-1', 'store-1', 'customer-1', '1299', 'paid')
on conflict (id) do nothing;

insert into public.sale_items (id, sale_id, sku, quantity)
values ('item-1', 'sale-1', 'sku-1', '1')
on conflict (id) do nothing;

insert into public.payments (id, sale_id, amount_cents, method)
values ('payment-1', 'sale-1', '1299', 'card')
on conflict (id) do nothing;
