CREATE TABLE programs (
	id      INTEGER NOT NULL PRIMARY KEY,
	name    VARCHAR NOT NULL UNIQUE
);

CREATE TABLE projects (
    id      INTEGER NOT NULL PRIMARY KEY,
	program INTEGER NOT NULL,
    name    VARCHAR NOT NULL,
    test    BLOB    NOT NULL,

	FOREIGN KEY (program) REFERENCES programs (id)
);
