CREATE TABLE employee (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL,
    role_id integer NOT NULL);

CREATE TABLE role (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL UNIQUE);

CREATE VIEW manager (id, name)
AS SELECT e.id, e.name
   FROM employee e
   JOIN role r ON e.role_id = r.id
   WHERE r.name = 'manager';
