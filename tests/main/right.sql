--
-- columns
--

-- column added

CREATE TABLE column_added (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b text NOT NULL,
	c integer NULL
);

-- column changed

CREATE TABLE column_changed (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b integer NULL
);

-- column ordinal changed

CREATE TABLE column_ordering_changed (
	b text NOT NULL,
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY
);

-- column removed

CREATE TABLE column_removed (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY
);

--
-- column privileges
--

-- column privilege added

CREATE TABLE column_privilege_added (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b text NOT NULL
);

CREATE ROLE column_privilege_added_role;

GRANT SELECT (a), INSERT (a), UPDATE (a)
ON TABLE column_privilege_added
TO column_privilege_added_role;

-- column privilege removed

CREATE TABLE column_privilege_removed (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b text NOT NULL
);

CREATE ROLE column_privilege_removed_role;

--
-- routines
--

CREATE TABLE routine (
	a integer PRIMARY KEY,
	b text NOT NULL
);

-- routine added

CREATE PROCEDURE routine_added(a integer, b text)
LANGUAGE SQL
AS
$$
INSERT INTO routine (a, b)
VALUES (a, b);
$$;

-- routine added with unnamed parameters

CREATE PROCEDURE routine_added_with_unnamed_parameters(integer, text)
LANGUAGE SQL
AS
$$
INSERT INTO routine (a, b)
VALUES ($1, $2);
$$;

-- routine changed

CREATE PROCEDURE routine_changed(a integer, b text)
LANGUAGE SQL
AS
$$
INSERT INTO routine (a, b)
VALUES (a + 100, b);
$$;

-- routine whitespace changed

CREATE PROCEDURE routine_whitespace_changed(a integer, b text)
LANGUAGE SQL
AS
$$
	INSERT INTO
		routine (a, b)
	VALUES
		(a, b);
$$;

-- routine removed

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

CREATE ROLE routine_privilege_added_role;

GRANT EXECUTE
ON PROCEDURE routine_privilege_added
TO routine_privilege_added_role;

-- routine privilege removed

CREATE PROCEDURE routine_privilege_removed(a integer, b text)
LANGUAGE SQL
AS
$$
INSERT INTO routine (a, b)
VALUES (a, b);
$$;

CREATE ROLE routine_privilege_removed_role;

--
-- schemas
--

-- schema added

CREATE SCHEMA schema_added;

-- schema changed

CREATE ROLE schema_changed_role;

CREATE SCHEMA schema_changed
AUTHORIZATION schema_changed_role;
CREATE SCHEMA schema_removed;

-- schema removed

--
-- sequences
--

-- sequence added

CREATE SEQUENCE sequence_added
AS bigint
INCREMENT BY 3
MINVALUE 30
MAXVALUE 99
START 30
NO CYCLE;

-- sequence changed

CREATE SEQUENCE sequence_changed
AS bigint
INCREMENT BY 3
MINVALUE 30
MAXVALUE 99
START 30
NO CYCLE;

-- sequence removed

--
-- tables
--

-- table added

CREATE TABLE table_added (
	a integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	b text NOT NULL
);

-- table changed

CREATE VIEW table_changed(a, b)
AS SELECT 1, 'changed';

-- table removed

--
-- table constraints
--

CREATE TABLE table_constraint (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY
);

-- table constraint added

CREATE TABLE table_constraint_added (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	a integer NOT NULL,
	CONSTRAINT fk_table_constraint_added FOREIGN KEY (a) REFERENCES table_constraint(id)
);

-- table constraint changed

CREATE TABLE table_constraint_changed (
	id integer GENERATED ALWAYS AS IDENTITY,
	a integer NOT NULL,
	CONSTRAINT my_constraint FOREIGN KEY (a) REFERENCES table_constraint(id)
);

-- table constraint removed

CREATE TABLE table_constraint_removed (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	a integer NOT NULL
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

GRANT SELECT, INSERT, UPDATE
ON TABLE table_privilege_added
TO table_privilege_added_role;

-- table privilege removed

CREATE TABLE table_privilege_removed (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	a integer NOT NULL
);

CREATE role table_privilege_removed_role;

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

CREATE FUNCTION trigger_exception()
RETURNS trigger AS
$$
	BEGIN
		RAISE EXCEPTION 'oh noes';
	END;
$$
LANGUAGE plpgsql;


-- trigger added

CREATE TRIGGER trigger_added
AFTER INSERT OR UPDATE OR DELETE
ON trigger_table
FOR EACH ROW
EXECUTE FUNCTION trigger_noop();

-- trigger changed

CREATE TRIGGER trigger_changed
AFTER INSERT OR UPDATE
ON trigger_table
FOR EACH ROW
WHEN (NEW.a = 666)
EXECUTE FUNCTION trigger_exception();

-- trigger removed

--
-- views
--

CREATE TABLE view_table (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	a integer NOT NULL
);

-- view added

CREATE VIEW view_added (id, name)
AS SELECT id, a
	FROM view_table
	WHERE id = 6;

-- view changed

CREATE VIEW view_changed (id, name)
AS SELECT a, id, 7 as seven
	FROM view_table
	WHERE a > 10;

-- view removed

