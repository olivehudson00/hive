CREATE TABLE users (
    id      INTEGER NOT NULL PRIMARY KEY,
    name    VARCHAR NOT NULL
);

CREATE TABLE programs (
    id      INTEGER NOT NULL PRIMARY KEY,
    name    VARCHAR NOT NULL
);

CREATE TABLE enrolments (
	id		INTEGER NOT NULL PRIMARY KEY,
	user    INTEGER NOT NULL,
	program INTEGER NOT NULL,

	FOREIGN KEY (user)
		REFERENCES users (id),
	FOREIGN KEY (program)
		REFERENCES program (id)
);

CREATE TABLE projects (
    id      INTEGER NOT NULL PRIMARY KEY,
    program INTEGER NOT NULL,
    name    VARCHAR NOT NULL,
    test    BLOB    NOT NULL,

    FOREIGN KEY (program)
        REFERENCES programs (id)
);

CREATE TABLE submissions (
    id      INTEGER NOT NULL PRIMARY KEY,
    user    INTEGER NOT NULL,
    project INTEGER NOT NULL,
	time    BIGINT  NOT NULL,
    results VARCHAR,

	FOREIGN KEY (user)
		REFERENCES users (id),
	FOREIGN KEY (project)
		REFERENCES projects (id)
);
