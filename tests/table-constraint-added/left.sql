CREATE TABLE role (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY
);

CREATE TABLE permission (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	role_id integer NOT NULL,
	CONSTRAINT fk_test FOREIGN KEY (role_id) REFERENCES role(id)
);

CREATE TABLE employee (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text,
	role_id integer NOT NULL
);
