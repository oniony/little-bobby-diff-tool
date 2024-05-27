CREATE TABLE role (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text UNIQUE
);

CREATE TABLE employee (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text,
    role_id integer NOT NULL,
    CONSTRAINT my_constraint FOREIGN KEY(role_id) REFERENCES role(id));
