CREATE TABLE employee (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL,
    role_id integer NOT NULL);

CREATE TABLE department (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL UNIQUE);

CREATE TABLE department_employee (
	department_id integer NOT NULL,
	employee_id integer NOT NULL,
	CONSTRAINT fk_department_id FOREIGN KEY(department_id) REFERENCES department(id),
	CONSTRAINT fk_employee_id FOREIGN KEY(employee_id) REFERENCES employee(id));

CREATE TABLE employee_role (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL UNIQUE);

CREATE VIEW leads (id, name)
AS SELECT e.id, e.name
   FROM employee e
   JOIN employee_role er ON e.role_id = er.id
   WHERE er.name = 'lead';

CREATE PROCEDURE new_employee(name text, role_id integer)
LANGUAGE SQL
AS $$
INSERT INTO employee(name, role_id)
VALUES (name, role_id);
$$;
