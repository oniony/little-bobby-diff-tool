CREATE TABLE employee (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL,
    role_id integer NOT NULL);

CREATE ROLE hagne;

GRANT SELECT, INSERT, UPDATE
ON TABLE employee
TO hagne;
