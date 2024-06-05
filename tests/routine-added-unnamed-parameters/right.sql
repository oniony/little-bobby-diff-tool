CREATE TABLE employee (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL,
    role_id integer NOT NULL);

CREATE PROCEDURE create_employee(integer, integer)
LANGUAGE SQL
AS
$$
INSERT INTO employee (name, role_id)
VALUES ($1, $2);
$$;
