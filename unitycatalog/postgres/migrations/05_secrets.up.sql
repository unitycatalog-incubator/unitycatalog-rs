create extension if not exists pgcrypto;

create table secrets (
  id uuid primary key default uuidv7 (),
  name Text collate case_insensitive not null,
  value bytea not null,
  created_at timestamptz not null default now (),
  updated_at timestamptz
);

select
  trigger_updated_at ('secrets');
