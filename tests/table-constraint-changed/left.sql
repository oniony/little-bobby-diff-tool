CREATE TABLE role (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text UNIQUE
);

CREATE TABLE employee (
	id integer GENERATED ALWAYS AS IDENTITY,
	name text,
    role_id integer NOT NULL,
    CONSTRAINT my_constraint PRIMARY KEY (id));
