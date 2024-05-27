CREATE TABLE employee (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL,
    role_id integer NOT NULL);

CREATE TABLE roles (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL UNIQUE);

CREATE PROCEDURE create_employee(name integer, role_name text)
LANGUAGE SQL
AS
$$
INSERT INTO employee (name, role_id)
VALUES (name, (SELECT id
	           FROM roles r
	           WHERE name = role_name));
$$;
