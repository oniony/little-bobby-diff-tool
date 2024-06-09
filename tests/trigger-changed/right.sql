CREATE TABLE employee (
	id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name text NOT NULL,
    role_id integer NOT NULL);

CREATE FUNCTION do_nothing()
RETURNS trigger AS
$$
	BEGIN
		RETURN NEW;
	END;
$$
LANGUAGE plpgsql;

CREATE FUNCTION do_something()
RETURNS trigger AS
$$
	BEGIN
		RAISE EXCEPTION 'oh noes';
	END;
$$

LANGUAGE plpgsql;
CREATE TRIGGER employee_added
BEFORE INSERT OR UPDATE ON employee
FOR EACH ROW
WHEN (NEW.role_id = 666)
EXECUTE FUNCTION do_something();
