CREATE TABLE users (
    id      INTEGER NOT NULL PRIMARY KEY,
    name    VARCHAR NOT NULL
);

CREATE TABLE programs (
    id      INTEGER NOT NULL PRIMARY KEY,
    name    VARCHAR NOT NULL
);

CREATE TABLE enrolments (
	id		   INTEGER NOT NULL PRIMARY KEY,
	user_id    INTEGER NOT NULL,
	program_id INTEGER NOT NULL,

	FOREIGN KEY (user_id)
		REFERENCES users (id),
	FOREIGN KEY (program_id)
		REFERENCES program (id)
);

CREATE TABLE projects (
    id         INTEGER NOT NULL PRIMARY KEY,
    program_id INTEGER NOT NULL,
    name       VARCHAR NOT NULL,
    test       BLOB    NOT NULL,
	grade      INTEGER NOT NULL,

    FOREIGN KEY (program_id)
        REFERENCES programs (id)
);

CREATE TABLE submissions (
    id         INTEGER NOT NULL PRIMARY KEY,
    user_id    INTEGER NOT NULL,
    project_id INTEGER NOT NULL,
	time       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    results    VARCHAR,
	grade      INTEGER,

	FOREIGN KEY (user_id)
		REFERENCES users (id),
	FOREIGN KEY (project_id)
		REFERENCES projects (id)
);
