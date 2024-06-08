CREATE TABLE employee (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL,
    role_id integer NOT NULL);

CREATE ROLE melitodes;

GRANT SELECT (name)
ON TABLE employee
TO melitodes;
