CREATE TABLE employee (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL,
    role_id integer NOT NULL);

CREATE VIEW manager (id, name)
AS SELECT id, name
   FROM employee
   WHERE role_id = 6;
