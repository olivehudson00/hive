/* shackle - seccomp jail
 * Copyright (C) 2025 Olive Hudson
 * see LICENCE file for licensing information */

#include <errno.h>
#include <fcntl.h>
#include <ftw.h>
#include <grp.h>
#include <pwd.h>
#include <seccomp.h>
#include <signal.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/prctl.h>
#include <sys/resource.h>
#include <sys/wait.h>
#include <unistd.h>

static void
die(const char *fmt, ...)
{
	va_list ap;
	va_start(ap, fmt);
	vfprintf(stderr, fmt, ap);
	va_end(ap);

	if (fmt[strlen(fmt) - 1] != '\n')
		perror(NULL);
	exit(127);
}

static int
tempcb(const char *path, const struct stat *sb, int typeflag,
		struct FTW *ftwbuf)
{
	return remove(path);
}

static int
proccb(const char *path, const struct stat *sb, int typeflag,
		struct FTW *ftwbuf)
{
	if (typeflag != FTW_F)
		return 0;

	int fd;
	sscanf(path, "/proc/self/fd/%d", &fd);
	return fd < 3 ? 0 : close(fd);
}

int
main(int argc, char **argv)
{
	if (argc < 2)
		die("usage: shackle [program] [args...]");

	/* get passwd file entry before */
	struct passwd *pw;
	if ((pw = getpwnam("hive")) == NULL)
		die("shackle: unable to get password file entry: ");

	/* create temporary folder to house program and later chroot to */
	char folder[] = "/tmp/hive.XXXXXX";
	if (mkdtemp(folder) == NULL)
		die("shackle: unable to make temporary directory: ");

	/* copy program to temporary folder */
	char file[] = "/tmp/hive.XXXXXX/program";
	memcpy(file, folder, sizeof(folder) - 1);

	int from, to;
	if ((from = open(argv[1], O_RDONLY)) == -1)
		die("shackle: unable to open file: ");
	if ((to = creat(file, 0555)) == -1)
		die("shackle: unable to create file: ");

	ssize_t nread;
	char buf[4096];
	while ((nread = read(from, buf, sizeof(buf))) > 0) {
		char *out = buf;
		ssize_t nwrite;

		do {
			nwrite = write(to, out, nread);

			if (nwrite >= 0) {
				nread -= nwrite;
				out += nwrite;
			} else if (errno != EINTR) {
				die("shackle: unable to write to file: ");
			}
		} while (nread > 0);
	}
	if (nread != 0)
		die("shackle: unable to read file: ");
	if (close(to))
		die("shackle: unable to close file: ");
	if (close(from))
		die("shackle: unable to close file: ");

	/* change file permissions on folder and file */
	if (chown(folder, pw->pw_uid, pw->pw_gid))
		die("shackle: unable to change folder ownership: ");
	if (chmod(folder, 0777))
		die("shackle: unable to change folder permissions: ");
	if (chown(file, pw->pw_uid, pw->pw_gid))
		die("shackle: unable to change file ownership: ");
	if (chmod(file, 0777))
		die("shackle: unable to change file permissions: ");

	/* close all open file descriptors except stdin, stdout, and stderr */
	if (nftw("/proc/self/fd", proccb, FOPEN_MAX, 0))
		die("shackle: unable to close open file descriptors: ");

	int pin[2], pout[2];
	if (pipe(pin) == -1)
		die("shackle: unable to create pipe: ");
	if (pipe(pout) == -1)
		die("shackle: unable to create pipe: ");

	/* fork and wait child */
	pid_t child;
	if ((child = fork()) != 0) {
		/* TODO: block SIGPIPE */

		if (child == -1)
			die("shackle: failed to fork process: ");

		int input = pin[1];
		int output = pout[0];

		if (close(pin[0]) == -1)
			die("shackle: unable to close file descriptor: ");
		if (close(pout[1]) == -1)
			die("shackle: unable to close file descriptor: ");

		int flags;
		if ((flags = fcntl(input, F_GETFL)) == -1)
			die("shackle: unable to get pipe flags: ");
		flags |= O_NONBLOCK;
		if (fcntl(input, F_SETFL, flags) == -1)
			die("shackle: unable to set pipe flags: ");
		if ((flags = fcntl(output, F_GETFL)) == -1)
			die("shackle: unable to get pipe flags: ");
		flags |= O_NONBLOCK;
		if (fcntl(output, F_SETFL, flags) == -1)
			die("shackle: unable to set pipe flags: ");





		sigset_t set;
		sigemptyset(&set);
		sigaddset(&set, SIGCHLD);
		siginfo_t info;
		struct timespec timeout = { .tv_sec = 5 };

		sigprocmask(SIG_BLOCK, &set, NULL);
		int result = sigtimedwait(&set, &info, &timeout);
		if (result == -1 && errno != EAGAIN)
			die("shackle: failed to wait for child process exit: ");

		/* kill the child process */
		kill(child, SIGKILL);
		if (waitpid(child, NULL, 0) == -1)
			die("shackle: unable to wait child process: ");

		/* remove temporary folder and contents */
		if (nftw(folder, tempcb, FOPEN_MAX, FTW_DEPTH | FTW_MOUNT | FTW_PHYS))
			die("shackle: unable to remove temporary folder and contents: ");

		return info.si_status;
	}

	/* enter chroot jail TODO */
	if (chdir(folder))
		die("shackle: unable to chdir into chroot directory: ");
	if (chroot("."))
		die("shackle: unable to chroot: ");
	if (chdir("."))
		die("shackle: unable to chdir into chroot directory: ");

	/* drop root privileges */
	if (initgroups("hive", pw->pw_gid))
		die("shackle: unable to init groups: ");
	if (setgid(pw->pw_gid))
		die("shackle: unable to set group id: ");
	if (setuid(pw->pw_uid))
		die("shackle: unable to set user id: ");

	/* restrict available memory and number of child processes */
	struct rlimit mem = { .rlim_cur = 1024 * 1024 * 2,
			.rlim_max = 1024 * 1024 * 2 };
	if (setrlimit(RLIMIT_AS, &mem))
		die("shackle: unable to restrict memory rlimit: ");
	struct rlimit proc = { .rlim_cur = 0, .rlim_max = 0 };
	if (setrlimit(RLIMIT_NPROC, &proc))
		die("shackle: unable to restrict child processes rlimit: ");

	/* create seccomp filter and only allow read, write, _exit, sigreturn,
	 * execve, and brk syscalls, as well as others needed for program
	 * instantiation */
	if (prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) == -1)
		die("shackle: unable to set no new privileges flag: ");

	scmp_filter_ctx ctx;
	if ((ctx = seccomp_init(SCMP_ACT_KILL)) == NULL)
		die("shackle: unable to init seccomp filter");

	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(read), 0))
		die("shackle: unable to add seccomp rule");
	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(write), 0))
		die("shackle: unable to add seccomp rule");
	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(exit_group), 0))
		die("shackle: unable to add seccomp rule");
	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(sigreturn), 0))
		die("shackle: unable to add seccomp rule");
	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(brk), 0))
		die("shackle: unable to add seccomp rule");
	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(execve), 0))
		die("shackle: unable to add seccomp rule");

	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(mprotect), 0))
		die("shackle: unable to add seccomp rule");
	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(readlinkat), 0))
		die("shackle: unable to add seccomp rule");
	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(getrandom), 0))
		die("shackle: unable to add seccomp rule");
	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(prlimit64), 0))
		die("shackle: unable to add seccomp rule");
	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(rseq), 0))
		die("shackle: unable to add seccomp rule");
	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(set_robust_list), 0))
		die("shackle: unable to add seccomp rule");
	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(set_tid_address), 0))
		die("shackle: unable to add seccomp rule");
	if (seccomp_rule_add(ctx, SCMP_ACT_ALLOW, SCMP_SYS(arch_prctl), 0))
		die("shackle: unable to add seccomp rule");

	if (seccomp_load(ctx))
		die("shackle: unable to load seccomp filter");

	char name[] = "program";
	argv[1] = name;
	execve(argv[1], argv, &(char *const){ NULL });
	die("shackle: unable to run program: ");
}
