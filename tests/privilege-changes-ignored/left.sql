--
-- table privileges
--

CREATE TABLE table_privilege_changes_ignored (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL,
    role_id integer NOT NULL);

CREATE ROLE table_privilege_changes_ignored_role;

--
-- column privileges
--

CREATE TABLE column_privilege_changes_ignored (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL,
    role_id integer NOT NULL);

CREATE ROLE column_privilege_changes_ignored_role;
