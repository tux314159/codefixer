#include <fcntl.h>
#include <seccomp.h>
#include <signal.h>
#include <stdint.h>
#include <stdlib.h>
#include <sys/resource.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

enum ChildStatus {
	AC = 0,
	WA = 1, /* not actually used here, but in the Rust grader */
	TLE = 2,
	MLE = 3,
	RTE = 4,
	UnknownError = 5,
};

struct ChildInfo {
	int64_t time_ms;
	int64_t max_mem_kib;
};

void
set_seccomp(scmp_filter_ctx *scmp_ctx)
{
	int allowed[] = {
		SCMP_SYS(access),
		SCMP_SYS(arch_prctl),
		SCMP_SYS(brk),
		SCMP_SYS(close),
		SCMP_SYS(execve),
		SCMP_SYS(exit_group),
		SCMP_SYS(fcntl),
		SCMP_SYS(fstat),
		SCMP_SYS(futex),
		SCMP_SYS(getcwd),
		SCMP_SYS(getdents64),
		SCMP_SYS(getegid),
		SCMP_SYS(geteuid),
		SCMP_SYS(getgid),
		SCMP_SYS(getrandom),
		SCMP_SYS(gettid),
		SCMP_SYS(getuid),
		SCMP_SYS(ioctl),
		SCMP_SYS(lseek),
		SCMP_SYS(mmap),
		SCMP_SYS(mprotect),
		SCMP_SYS(munmap),
		SCMP_SYS(newfstatat),
		SCMP_SYS(open),
		SCMP_SYS(openat),
		SCMP_SYS(prlimit64),
		SCMP_SYS(read),
		SCMP_SYS(readlink),
		SCMP_SYS(rseq),
		SCMP_SYS(rt_sigaction),
		SCMP_SYS(rt_sigprocmask),
		SCMP_SYS(sched_getaffinity),
		SCMP_SYS(set_robust_list),
		SCMP_SYS(set_tid_address),
		SCMP_SYS(write),
		-1
	};
	for (int *sys = (int *)allowed; *sys != -1; sys++)
		seccomp_rule_add(*scmp_ctx, SCMP_ACT_ALLOW, *sys, 0);
}

enum ChildStatus
run_limited(
	const char *__restrict__ exe_path,
	const char *__restrict__ input_path,
	const char *__restrict__ output_path,
	struct ChildInfo *info,
	int64_t tl_ms,
	int64_t ml_kib
)
{
	int ppe[2];
	int ip_fd, op_fd;
	pid_t pid;

	pipe(ppe);

	if ((ip_fd = open(input_path, O_RDONLY)) == -1)
		return UnknownError;
	if ((op_fd = open(
			 output_path,
			 O_CREAT | O_WRONLY,
			 S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH
		 )) == -1) {
		close(ip_fd);
		return UnknownError;
	}
	ftruncate(op_fd, 0); /* for testing */

	pid = fork();
	if (pid) {
		int cinfo;
		int64_t tmbuf;
		struct timespec sleeptime = {
			.tv_sec = 0,
			.tv_nsec = 2e7L
		}; /* poll every 20ms */
		struct timespec starttime;
		struct timespec curtime;
		struct rusage rusage;
		time_t tdiff;

		enum ChildStatus child_status = UnknownError;

		read(ppe[0], &tmbuf, 8);
		starttime.tv_sec = tmbuf;
		read(ppe[0], &tmbuf, 8);
		starttime.tv_nsec = tmbuf;

		while (waitpid(pid, &cinfo, WNOHANG) == 0) {
			getrusage(RUSAGE_CHILDREN, &rusage);
			clock_gettime(CLOCK_MONOTONIC, &curtime);
			tdiff =
				((curtime.tv_nsec - starttime.tv_nsec) +
			     (curtime.tv_sec - starttime.tv_sec) * 1e9);
			if (tdiff > 2 * tl_ms * 1e6) {
				kill(pid, SIGKILL);
				child_status = TLE;
			} else if (rusage.ru_maxrss > ml_kib * 2) {
				kill(pid, SIGKILL);
				child_status = MLE;
			}

			nanosleep(&sleeptime, NULL);
		}

		getrusage(RUSAGE_CHILDREN, &rusage);
		clock_gettime(CLOCK_MONOTONIC, &curtime);
		tdiff =
			((curtime.tv_nsec - starttime.tv_nsec) +
		     (curtime.tv_sec - starttime.tv_sec) * 1e9);

		info->time_ms = tdiff / 1e6;
		info->max_mem_kib = rusage.ru_maxrss;
		if (child_status != UnknownError) {
			return child_status;
		} else {
			if (tdiff > tl_ms * 1e6)
				return TLE;
			else if (rusage.ru_maxrss > ml_kib)
				return MLE;
			else if (WEXITSTATUS(cinfo) == 0)
				return AC;
			else
				return UnknownError;
		}

	} else {
		scmp_filter_ctx scmp_ctx;
		int64_t tmbuf;
		char *cmd[] = {NULL};
		char *env[] = {NULL};
		struct rlimit cpu_lim;
		struct rlimit core_lim = {0, 0};
		struct timespec starttime;

		dup2(ip_fd, STDIN_FILENO);
		dup2(op_fd, STDOUT_FILENO);

		cpu_lim.rlim_cur = 2 * tl_ms / 1000;
		cpu_lim.rlim_max = cpu_lim.rlim_cur;
		setrlimit(
			RLIMIT_CPU,
			&cpu_lim
		); /* just in case of funny thread business */
		setrlimit(RLIMIT_CORE, &core_lim); /* disable coredumps */

		scmp_ctx = seccomp_init(SCMP_ACT_ERRNO(100));
		if (scmp_ctx == NULL)
			_exit(1);
		set_seccomp(&scmp_ctx);
		if (seccomp_load(scmp_ctx))
			_exit(1);

		/* Send start time to parent. */
		clock_gettime(CLOCK_MONOTONIC, &starttime);
		tmbuf = starttime.tv_sec;
		write(ppe[1], &tmbuf, 8);
		tmbuf = starttime.tv_nsec;
		write(ppe[1], &tmbuf, 8);

		execve(exe_path, cmd, env);
		_exit(1); /* if exec failed */
	}
};
