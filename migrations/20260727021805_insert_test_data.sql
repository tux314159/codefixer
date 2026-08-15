INSERT INTO users (id, username, google_id, email)
	VALUES (1, "tux", "105696583958146482877", "tux@tux.tux");
INSERT INTO users (id, username, google_id, email)
	VALUES (2, "fake tux", "105696583958146482878", "fake@tux.tux");

INSERT INTO problems (id, title, source, tl, ml, runtype, created_at)
	VALUES (1, "Addition of 2 numbers", "Classic problem", 1000, 1024, 0, 1786775085);

INSERT INTO problem_authors (FK_problems_id, FK_users_id)
	VALUES (1, 1);
INSERT INTO problem_authors (FK_problems_id, FK_users_id)
	VALUES (1, 2);

INSERT INTO problem_tags (FK_problems_id, tag)
	VALUES (1, "syntax");

INSERT INTO subtasks (FK_problems_id, subtask, score)
	VALUES (1, 1, 0);
INSERT INTO subtasks (FK_problems_id, subtask, score)
	VALUES (1, 2, 50);
INSERT INTO subtasks (FK_problems_id, subtask, score)
	VALUES (1, 3, 50);

INSERT INTO subtask_testcases (FK_subtasks_problems_id, FK_subtasks_subtask, testcase)
	VALUES (1, 1, 1);
INSERT INTO subtask_testcases (FK_subtasks_problems_id, FK_subtasks_subtask, testcase)
	VALUES (1, 2, 1);
INSERT INTO subtask_testcases (FK_subtasks_problems_id, FK_subtasks_subtask, testcase)
	VALUES (1, 2, 2);
INSERT INTO subtask_testcases (FK_subtasks_problems_id, FK_subtasks_subtask, testcase)
	VALUES (1, 3, 1);
INSERT INTO subtask_testcases (FK_subtasks_problems_id, FK_subtasks_subtask, testcase)
	VALUES (1, 3, 2);
INSERT INTO subtask_testcases (FK_subtasks_problems_id, FK_subtasks_subtask, testcase)
	VALUES (1, 3, 3);
INSERT INTO subtask_testcases (FK_subtasks_problems_id, FK_subtasks_subtask, testcase)
	VALUES (1, 3, 4);

INSERT INTO submissions (id, FK_users_id, FK_problems_id, language, timestamp, score)
	VALUES (1, 1, 1, "C++", 0, 50);

INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status)
	VALUES (1, 1, 100, 100, 0, 0);
INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status)
	VALUES (1, 2, 100, 100, 0, 0);
INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status)
	VALUES (1, 3, 100, 100, 0, 0);
INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status)
	VALUES (1, 4, 100, 100, 0, 1);

INSERT INTO submissions (id, FK_users_id, FK_problems_id, language, timestamp, score)
	VALUES (2, 2, 1, "C++", 0, 100);

INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status)
	VALUES (2, 1, 100, 100, 0, 0);
INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status)
	VALUES (2, 2, 100, 100, 0, 0);
INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status)
	VALUES (2, 3, 100, 100, 0, 0);
INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status)
	VALUES (2, 4, 100, 100, 0, 0);

INSERT INTO submissions (id, FK_users_id, FK_problems_id, language, timestamp, score)
	VALUES (3, 2, 1, "python", 0, 100);

INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status)
	VALUES (3, 1, 999, 420, 0, 0);
INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status)
	VALUES (3, 2, 999, 420, 0, 0);
INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status)
	VALUES (3, 3, 999, 420, 0, 0);
INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status)
	VALUES (3, 4, 999, 420, 0, 0);

