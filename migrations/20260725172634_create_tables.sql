CREATE TABLE users (
	id INTEGER NOT NULL UNIQUE,
	username VARCHAR(32) NOT NULL UNIQUE,
	google_id VARCHAR(256) NOT NULL UNIQUE,
	email VARCHAR(256) NOT NULL UNIQUE,
	PRIMARY KEY(id AUTOINCREMENT)
);

CREATE TABLE problems (
	id INTEGER NOT NULL UNIQUE,
	title VARCHAR(64) NOT NULL,
	source VARCHAR(64) NOT NULL,
	tl INTEGER NOT NULL,
	ml INTEGER NOT NULL,
	runtype INTEGER NOT NULL,
	created_at INTEGER NOT NULL,
	PRIMARY KEY(id AUTOINCREMENT)
);

CREATE TABLE problem_authors (
	FK_problems_id INTEGER NOT NULL,
	FK_users_id INTEGER NOT NULL,
	PRIMARY KEY(FK_problems_id, FK_users_id),
	FOREIGN KEY(FK_problems_id) REFERENCES problems(id),
	FOREIGN KEY(FK_users_id) REFERENCES users(id)
);

CREATE TABLE problem_tags (
	FK_problems_id INTEGER NOT NULL,
	tag VARCHAR(64) NOT NULL,
	PRIMARY KEY(FK_problems_id, tag),
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
	FK_subtasks_problems_id INTEGER NOT NULL,
	FK_subtasks_subtask INTEGER NOT NULL,
	testcase INTEGER NOT NULL,
	PRIMARY KEY(FK_subtasks_problems_id, FK_subtasks_subtask, testcase),
	FOREIGN KEY(FK_subtasks_problems_id, FK_subtasks_subtask) REFERENCES subtasks(FK_problems_id, subtask)
);

-- This table is not normalised (score) because of performance issues doing a
-- four-way join for a few million rows on every page load.
CREATE TABLE submissions (
	id INTEGER UNIQUE NOT NULL,
	FK_users_id INTEGER NOT NULL,
	FK_problems_id INTEGER NOT NULL,
	language VARCHAR(32) NOT NULL,
	timestamp INTEGER NOT NULL,
	score INTEGER NOT NULL,
	PRIMARY KEY(id AUTOINCREMENT),
	FOREIGN KEY(FK_users_id) REFERENCES users(id),
	FOREIGN KEY(FK_problems_id) REFERENCES problems(id)
);
	
-- This table is not normalised because status does not change even if problem
-- limits have changed until a regrade.
CREATE TABLE submission_testcases (
	FK_submissions_id INTEGER NOT NULL,
	testcase INTEGER NOT NULL,
	max_time INTEGER NOT NULL,
	max_mem INTEGER NOT NULL,
	exit_code INTEGER NOT NULL,
	status INTEGER NOT NULL, -- enum: 0 is OK, 1 is TLE, 2 is MLE, 3 is RTE
	PRIMARY KEY(FK_submissions_id, testcase),
	FOREIGN KEY(FK_submissions_id) REFERENCES submissions(id)
);

CREATE TABLE oauth_tokens (
	token VARCHAR(2048) NOT NULL UNIQUE,
	verifier VARCHAR(2048) NOT NULL,
	PRIMARY KEY(token)
);
