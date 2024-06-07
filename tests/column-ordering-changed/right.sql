CREATE TABLE employee (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    role_id integer NOT NULL,
	name text NOT NULL);
