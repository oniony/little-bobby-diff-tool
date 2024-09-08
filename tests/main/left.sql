--
-- columns
--

-- column added

CREATE TABLE column_added (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b text NOT NULL
);

-- column changed

CREATE TABLE column_changed (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b text NOT NULL
);

-- column ordinal changed

CREATE TABLE column_ordering_changed (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b text NOT NULL
);

-- column removed

CREATE TABLE column_removed (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b text NOT NULL
);

-- column privilege added

CREATE TABLE column_privilege_added (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b text NOT NULL
);

CREATE ROLE column_privilege_added_role;

-- column privilege removed

CREATE TABLE column_privilege_removed (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b text NOT NULL
);

CREATE ROLE column_privilege_removed_role;

GRANT SELECT (a), INSERT (a), UPDATE (a)
ON TABLE column_privilege_removed
TO column_privilege_removed_role;

-- common routine table

CREATE TABLE routine (
	a integer PRIMARY KEY,
	b text NOT NULL
);

-- routine added

-- routine added with unnamed parameters

-- routine changed

CREATE PROCEDURE routine_changed(a integer, b text)
LANGUAGE SQL
AS
$$
INSERT INTO routine (a, b)
VALUES (a, b);
$$;

-- routine whitespace changed

CREATE PROCEDURE routine_whitespace_changed(a integer, b text)
LANGUAGE SQL
AS
$$
INSERT INTO routine (a, b)
VALUES (a, b);
$$;

-- routine removed

CREATE PROCEDURE routine_removed(a integer, b text)
LANGUAGE SQL
AS
$$
INSERT INTO routine (a, b)
VALUES (a, b);
$$;

--
-- routine privileges
--

-- routine privilege added

CREATE PROCEDURE routine_privilege_added(a integer, b text)
LANGUAGE SQL
AS
$$
INSERT INTO routine (a, b)
VALUES (a, b);
$$;

-- routine privilege removed

CREATE PROCEDURE routine_privilege_removed(a integer, b text)
LANGUAGE SQL
AS
$$
INSERT INTO routine (a, b)
VALUES (a, b);
$$;

CREATE ROLE routine_privilege_removed_role;

GRANT EXECUTE
ON PROCEDURE routine_privilege_removed
TO routine_privilege_removed_role;

--
-- schemas
--

-- schema added

-- schema changed

CREATE ROLE schema_changed_role;

CREATE SCHEMA schema_changed;

-- schema removed

CREATE SCHEMA schema_removed;

--
-- sequences
--

-- sequence added

-- sequence changed

CREATE SEQUENCE sequence_changed
AS integer
INCREMENT BY 2
MINVALUE 10
MAXVALUE 100
START 44
CYCLE;

-- sequence removed

CREATE SEQUENCE sequence_removed
AS bigint
INCREMENT BY 3
MINVALUE 30
MAXVALUE 99
START 30
NO CYCLE;

--
-- tables
--

-- table added

-- table changed

CREATE TABLE table_changed (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b text NOT NULL
);

-- table removed

CREATE TABLE table_removed (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b text NOT NULL
);

--
-- table constraints
--

CREATE TABLE table_constraint (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY
);

-- table constraint added

CREATE TABLE table_constraint_added (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	a integer NOT NULL
);

-- table constraint changed

CREATE TABLE table_constraint_changed (
	id integer GENERATED ALWAYS AS IDENTITY,
	a integer NOT NULL,
	CONSTRAINT my_constraint PRIMARY KEY (id)
);

-- table constraint removed

CREATE TABLE table_constraint_removed (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	a integer NOT NULL,
	CONSTRAINT fk_table_constraint_removed FOREIGN KEY (a) REFERENCES table_constraint(id)
);

--
-- table privileges
--

-- table privilege added

CREATE TABLE table_privilege_added (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	a integer NOT NULL
);

CREATE role table_privilege_added_role;

-- table privilege removed

CREATE TABLE table_privilege_removed (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	a integer NOT NULL
);

CREATE role table_privilege_removed_role;

GRANT SELECT, INSERT, UPDATE
ON TABLE table_privilege_removed
TO table_privilege_removed_role;

--
-- triggers
--

CREATE TABLE trigger_table (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	a integer NOT NULL
);

CREATE FUNCTION trigger_noop()
RETURNS trigger AS
$$
	BEGIN
		RETURN NEW;
	END;
$$
LANGUAGE plpgsql;

-- trigger added

-- trigger changed

CREATE TRIGGER trigger_changed
AFTER INSERT OR UPDATE OR DELETE
ON trigger_table
FOR EACH ROW
EXECUTE FUNCTION trigger_noop();

-- trigger removed

CREATE TRIGGER trigger_removed
AFTER INSERT OR UPDATE OR DELETE
ON trigger_table
FOR EACH ROW
EXECUTE FUNCTION trigger_noop();

--
-- views
--

CREATE TABLE view_table (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	a integer NOT NULL
);

-- view added

-- view changed

CREATE VIEW view_changed (id, name)
AS SELECT id, a
	FROM view_table
	WHERE id = 6;

-- view removed

CREATE VIEW view_removed (id, name)
AS SELECT id, a
	FROM view_table
	WHERE id = 6;
