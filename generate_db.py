#!/usr/bin/python

import os
import random
import sqlite3

import faker

N_USERS = 5000
N_PROBLEMS = 3000
N_SUBMISSIONS = 1000000  # 1000000
N_TESTCASES = 32  # 64

fake = faker.Faker()

with open("migrations/20260725172634_create_tables.sql", "r") as f:
    create_tables_sql = f.read()

try:
    os.remove("state/db.sqlite3")
except:  # noqa: E722, S110
    pass

fake = faker.Faker()

print("Generating users...")
users = [(1, "tux", "105696583958146482877", "tux@tux.tux", 1)]
for user_id in range(2, N_USERS + 1):
    fname = fake.first_name().lower()
    lname = fake.last_name().lower()
    nonce = pow(user_id, 67, 2**31 - 1)
    name = f"{fname}_{nonce}"
    gid = fake.unique.aba()
    email = f"{fname}.{lname}@{nonce}.example.com"
    user = (user_id, name, str(gid), email, 1)
    users.append(user)

problems = []
problem_authors = []
subtasks = []
submissions = []
testcases = []
subtask_testcases = []
print("Generating problems...")
for prob_id in range(1, N_PROBLEMS + 1):
    # Generate subtasks.
    n_st = 1
    r = 100
    s = [0]
    while True:
        ss = random.randint(10, 60)
        if r - ss < 1:
            break
        r -= ss
        s.append(ss)
        n_st += 1
    s.append(r)
    n_st += 1
    s.append(r)

    for st in range(1, n_st + 1):
        subtasks.append((prob_id, st, s[st - 1]))

    # Insert relations.
    idxtcs = list(range(N_TESTCASES))
    for st in range(1, n_st + 1):
        for tc in (
            l := random.sample(
                list(range(1, N_TESTCASES + 1)), random.randint(1, N_TESTCASES)
            )
        ):
            subtask_testcases.append((prob_id, st, tc))

    # Generate problems.
    title = " ".join([s.title() for s in fake.words(3)])
    runtype = random.choice([1, 2, 3])
    source = random.choice(["Classic Problem", "Dunjudge Archive", "NOI 2030"])
    tl = random.choice([1000, 2000])
    ml = random.choice([1024, 2048])

    problem = (prob_id, title, source, tl, ml, runtype, 1786780388)
    problems.append(problem)

    # Generate author relations.
    n_auth = random.randint(1, 3)
    authors = random.sample(range(1, N_USERS + 1), n_auth)
    for a in authors:
        problem_authors.append((prob_id, a))

# Generate submissions.
subs = []
sub_tcs = []
print("Generating submissions...")
for sub_id in range(1, N_SUBMISSIONS + 1):
    user = random.randint(1, N_USERS)
    problem = random.randint(1, N_PROBLEMS)
    timestamp = random.randint(0, 2**16)
    timestamp = random.randint(0, 2**31)

    tl = problems[problem - 1][3]
    ml = problems[problem - 1][4]
    s = 0
    for tc in range(1, N_TESTCASES + 1):
        time = random.randint(1, tl + 10)
        mem = random.randint(1, ml + 10)
        status = 0
        if time > tl:
            status = 1
        elif mem > ml:
            status = 2
        s += status
        sub_tcs.append((sub_id, tc, time, mem, 0, status))

    score = 100 if s == 0 else (0 if random.random() > 0.7 else random.randint(1, 99))
    language = random.choice([1, 2, 3])
    subs.append((sub_id, user, problem, language, timestamp, score))

    # Connect to an existing database
with sqlite3.connect("state/db.sqlite3") as conn:
    cur = conn.cursor()
    print("Creating tables")
    cur.executescript(create_tables_sql)

    print("Inserting users")
    cur.executemany(
        "INSERT INTO users (id, username, google_id, email, role) VALUES (?, ?, ?, ?, ?)",
        users,
    )
    print("Inserting problems")
    cur.executemany(
        "INSERT INTO problems (id, title, source, tl, ml, runtype, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        problems,
    )
    print("Inserting authors")
    cur.executemany(
        "INSERT INTO problem_authors (FK_problems_id, FK_users_id) VALUES (?, ?)",
        problem_authors,
    )
    print("Inserting subtasks")
    cur.executemany(
        "INSERT INTO subtasks (FK_problems_id, subtask, score) VALUES (?, ?, ?)",
        subtasks,
    )
    print("Inserting subtask-testcase links")
    cur.executemany(
        "INSERT INTO subtask_testcases (FK_subtasks_problems_id, FK_subtasks_subtask, testcase) VALUES (?,?,?)",
        subtask_testcases,
    )
    conn.commit()
    print("Inserting submissions")
    cur.executemany(
        "INSERT INTO submissions (id, FK_users_id, FK_problems_id, language, timestamp, score) VALUES (?, ?, ?, ?, ?, ?)",
        subs,
    )
    conn.commit()
    print("Inserting submission-testcase links")
    cur.executemany(
        "INSERT INTO submission_testcases (FK_submissions_id, testcase, max_time, max_mem, exit_code, status) VALUES (?, ?, ?, ?, ?, ?)",
        sub_tcs,
    )

    cur.close()
    conn.commit()

"""
"""
