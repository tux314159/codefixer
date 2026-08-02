CREATE TABLE users (
	id INTEGER NOT NULL UNIQUE,
	username VARCHAR(32) NOT NULL UNIQUE,
	google_id VARCHAR(256) NOT NULL UNIQUE,
	email VARCHAR(256) NOT NULL UNIQUE,
	PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE problems (
	id INTEGER NOT NULL UNIQUE,
	name VARCHAR(64) NOT NULL,
	source VARCHAR(64) NOT NULL,
	tl INTEGER NOT NULL,
	ml INTEGER NOT NULL,
	runtype INTEGER NOT NULL,
	PRIMARY KEY(id AUTOINCREMENT)
);

CREATE TABLE testcases (
	FK_problems_id INTEGER NOT NULL,
	testcase INTEGER NOT NULL UNIQUE,
	PRIMARY KEY(FK_problems_id, testcase),
	FOREIGN KEY(FK_problems_id) REFERENCES problems(id)
);

CREATE TABLE subtasks (
	FK_problems_id INTEGER NOT NULL,
	subtask INTEGER NOT NULL,
	score INTEGER NOT NULL,
	PRIMARY KEY(FK_problems_id, subtask),
	FOREIGN KEY(FK_problems_id) REFERENCES problems(id)
);

CREATE TABLE subtask_testcases (
	FK_problems_id INTEGER NOT NULL,
	FK_subtasks_subtask INTEGER NOT NULL,
	FK_testcases_testcase INTEGER NOT NULL,
	PRIMARY KEY(FK_problems_id, FK_subtasks_subtask, FK_testcases_testcase),
	FOREIGN KEY(FK_problems_id) REFERENCES problems(id),
	FOREIGN KEY(FK_subtasks_subtask) REFERENCES subtasks(subtask),
	FOREIGN KEY(FK_testcases_testcase) REFERENCES testcases(testcase)
);

CREATE TABLE submissions (
	id INTEGER UNIQUE NOT NULL,
	FK_users_id INTEGER NOT NULL,
	FK_problems_id INTEGER NOT NULL,
	language VARCHAR(32) NOT NULL,
	timestamp INTEGER NOT NULL,
	max_time INTEGER NOT NULL,
	max_mem INTEGER NOT NULL,
	PRIMARY KEY(id AUTOINCREMENT),
	FOREIGN KEY(FK_users_id) REFERENCES users(id),
	FOREIGN KEY(FK_problems_id) REFERENCES problems(id)
);
	

CREATE TABLE submission_testcases (
	FK_submissions_id INTEGER NOT NULL,
	FK_testcases_testcase INTEGER NOT NULL,
	status INTEGER NOT NULL,  -- enum: 0 is ok, 1 is TLE, 2 is MLE, 3 is RTE
	PRIMARY KEY(FK_submissions_id, FK_testcases_testcase),
	FOREIGN KEY(FK_submissions_id) REFERENCES submissions(id),
	FOREIGN KEY(FK_testcases_testcase) REFERENCES testcases(testcase)
);

CREATE TABLE oauth_tokens (
	token VARCHAR(2048) NOT NULL UNIQUE,
	verifier VARCHAR(2048) NOT NULL,
	PRIMARY KEY(token)
);
