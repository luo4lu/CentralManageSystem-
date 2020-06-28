create database test_centralmanagesystem;

create table agents(
    id varchar(255) PRIMARY KEY NOT NULL,
    cert text NOT NULL,
    extra jsonb NOT NULL,
    state varchar(255) NOT NULL,
    create_time timestamp NOT NULL,
    update_time timestamp NOT NULL,
    type varchar(255) NOT NULL);

create table quota_admin(
    id varchar(255) PRIMARY KEY NOT NULL,
    aid varchar(255) REFERENCES agents (id),
    extra jsonb NOT NULL,
    value bigint NOT NULL,
    type varchar(255) NOT NULL,
    state varchar(255) NOT NULL,
    create_time timestamp NOT NULL,
    update_time timestamp NOT NULL
);

create table quota_delivery(
    id varchar(255) PRIMARY KEY NOT NULL,
    aid varchar(255) REFERENCES agents (id) NOT NULL,
    issue text NOT NULL,
    issue_info jsonb NOT NULL,
    create_time timestamp NOT NULL,
    update_time timestamp NOT NULL
);