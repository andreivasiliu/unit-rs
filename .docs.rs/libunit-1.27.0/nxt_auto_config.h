
#ifndef NXT_GCC
#define NXT_GCC  1
#endif


#define NXT_CONFIGURE_OPTIONS  " --prefix=/usr --state=/var/lib/unit --control=unix:/var/run/control.unit.sock --pid=/var/run/unit.pid --log=/var/log/unit.log --tmp=/var/tmp --user=unit --group=unit --tests --openssl --modules=/usr/lib/unit/modules --libdir=/usr/lib/x86_64-linux-gnu --cc-opt='-g -O2 -ffile-prefix-map=/data/builder/debuild/unit-1.27.0/pkg/deb/debuild/unit-1.27.0=. -flto=auto -ffat-lto-objects -flto=auto -ffat-lto-objects -specs=/usr/share/dpkg/no-pie-compile.specs -fstack-protector-strong -Wformat -Werror=format-security -Wp,-D_FORTIFY_SOURCE=2 -fPIC' --ld-opt='-Wl,-Bsymbolic-functions -flto=auto -ffat-lto-objects -flto=auto -specs=/usr/share/dpkg/no-pie-link.specs -Wl,-z,relro -Wl,-z,now -Wl,--as-needed -pie'"
#define NXT_SYSTEM_VERSION     "Linux 5.15.0-1004-aws x86_64"
#define NXT_COMPILER_VERSION   "gcc version 11.2.0 (Ubuntu 11.2.0-19ubuntu1) "

#define NXT_PID                "/var/run/unit.pid"
#define NXT_LOG                "/var/log/unit.log"
#define NXT_MODULES            "/usr/lib/unit/modules"
#define NXT_STATE              "/var/lib/unit"
#define NXT_TMP                "/var/tmp"

#define NXT_CONTROL_SOCK       "unix:/var/run/control.unit.sock"

#define NXT_USER               "unit"
#define NXT_GROUP              "unit"


#ifndef NXT_UNIX
#define NXT_UNIX  1
#endif


#ifndef NXT_HAVE_UNIX_DOMAIN
#define NXT_HAVE_UNIX_DOMAIN  1
#endif


#ifndef NXT_INT_SIZE
#define NXT_INT_SIZE  4
#endif


#ifndef NXT_LONG_SIZE
#define NXT_LONG_SIZE  8
#endif


#ifndef NXT_LONG_LONG_SIZE
#define NXT_LONG_LONG_SIZE  8
#endif


#ifndef NXT_PTR_SIZE
#define NXT_PTR_SIZE  8
#endif


#ifndef NXT_SIZE_T_SIZE
#define NXT_SIZE_T_SIZE  8
#endif


#ifndef NXT_OFF_T_SIZE
#define NXT_OFF_T_SIZE  8
#endif


#ifndef NXT_TIME_T_SIZE
#define NXT_TIME_T_SIZE  8
#endif


#ifndef NXT_HAVE_C99_VARIADIC_MACRO
#define NXT_HAVE_C99_VARIADIC_MACRO  1
#endif


#ifndef NXT_HAVE_BUILTIN_EXPECT
#define NXT_HAVE_BUILTIN_EXPECT  1
#endif


#ifndef NXT_HAVE_BUILTIN_UNREACHABLE
#define NXT_HAVE_BUILTIN_UNREACHABLE  1
#endif


#ifndef NXT_HAVE_BUILTIN_PREFETCH
#define NXT_HAVE_BUILTIN_PREFETCH  1
#endif


#ifndef NXT_HAVE_BUILTIN_CLZ
#define NXT_HAVE_BUILTIN_CLZ  1
#endif


#ifndef NXT_HAVE_BUILTIN_POPCOUNT
#define NXT_HAVE_BUILTIN_POPCOUNT  1
#endif


#ifndef NXT_HAVE_GCC_ATTRIBUTE_VISIBILITY
#define NXT_HAVE_GCC_ATTRIBUTE_VISIBILITY  1
#endif


#ifndef NXT_HAVE_GCC_ATTRIBUTE_ALIGNED
#define NXT_HAVE_GCC_ATTRIBUTE_ALIGNED  1
#endif


#ifndef NXT_HAVE_GCC_ATTRIBUTE_MALLOC
#define NXT_HAVE_GCC_ATTRIBUTE_MALLOC  1
#endif


#ifndef NXT_HAVE_GCC_ATTRIBUTE_PACKED
#define NXT_HAVE_GCC_ATTRIBUTE_PACKED  1
#endif


#ifndef NXT_HAVE_GCC_ATTRIBUTE_UNUSED
#define NXT_HAVE_GCC_ATTRIBUTE_UNUSED  1
#endif


#ifndef NXT_HAVE_GCC_ATOMIC
#define NXT_HAVE_GCC_ATOMIC  1
#endif


#ifndef NXT_HAVE_POSIX_MEMALIGN
#define NXT_HAVE_POSIX_MEMALIGN  1
#endif


#ifndef NXT_HAVE_MALLOC_USABLE_SIZE
#define NXT_HAVE_MALLOC_USABLE_SIZE  1
#endif


#ifndef NXT_HAVE_ALLOCA
#define NXT_HAVE_ALLOCA  1
#endif


#ifndef NXT_HAVE_MALLOPT
#define NXT_HAVE_MALLOPT  1
#endif


#ifndef NXT_HAVE_MAP_ANON
#define NXT_HAVE_MAP_ANON  1
#endif


#ifndef NXT_HAVE_MAP_POPULATE
#define NXT_HAVE_MAP_POPULATE  1
#endif


#ifndef NXT_HAVE_SHM_OPEN
#define NXT_HAVE_SHM_OPEN  1
#endif


#ifndef NXT_HAVE_MEMFD_CREATE
#define NXT_HAVE_MEMFD_CREATE  1
#endif


#ifndef NXT_HAVE_CLOCK_REALTIME_COARSE
#define NXT_HAVE_CLOCK_REALTIME_COARSE  1
#endif


#ifndef NXT_HAVE_CLOCK_REALTIME
#define NXT_HAVE_CLOCK_REALTIME  1
#endif


#ifndef NXT_HAVE_CLOCK_MONOTONIC_COARSE
#define NXT_HAVE_CLOCK_MONOTONIC_COARSE  1
#endif


#ifndef NXT_HAVE_CLOCK_MONOTONIC
#define NXT_HAVE_CLOCK_MONOTONIC  1
#endif


#ifndef NXT_HAVE_TM_GMTOFF
#define NXT_HAVE_TM_GMTOFF  1
#endif


#ifndef NXT_HAVE_LOCALTIME_R
#define NXT_HAVE_LOCALTIME_R  1
#endif


#ifndef NXT_HAVE_PTHREAD_SPINLOCK
#define NXT_HAVE_PTHREAD_SPINLOCK  1
#endif


#ifndef NXT_HAVE_SEM_TIMEDWAIT
#define NXT_HAVE_SEM_TIMEDWAIT  1
#endif


#ifndef NXT_HAVE_THREAD_STORAGE_CLASS
#define NXT_HAVE_THREAD_STORAGE_CLASS  1
#endif


#ifndef NXT_HAVE_EPOLL
#define NXT_HAVE_EPOLL  1
#endif


#ifndef NXT_HAVE_SIGNALFD
#define NXT_HAVE_SIGNALFD  1
#endif


#ifndef NXT_HAVE_EVENTFD
#define NXT_HAVE_EVENTFD  1
#endif


#ifndef NXT_INET6
#define NXT_INET6  1
#endif


#ifndef NXT_HAVE_SOCKADDR
#define NXT_HAVE_SOCKADDR  16
#endif


#ifndef NXT_HAVE_SOCKADDR_IN
#define NXT_HAVE_SOCKADDR_IN  16
#endif


#ifndef NXT_HAVE_SOCKADDR_IN6
#define NXT_HAVE_SOCKADDR_IN6  28
#endif


#ifndef NXT_HAVE_SOCKADDR_UN
#define NXT_HAVE_SOCKADDR_UN  110
#endif


#ifndef NXT_HAVE_SOCKADDR_STORAGE
#define NXT_HAVE_SOCKADDR_STORAGE  128
#endif


#ifndef NXT_HAVE_AF_UNIX_SOCK_SEQPACKET
#define NXT_HAVE_AF_UNIX_SOCK_SEQPACKET  1
#endif


#ifndef NXT_HAVE_MSGHDR_MSG_CONTROL
#define NXT_HAVE_MSGHDR_MSG_CONTROL  1
#endif


#ifndef NXT_HAVE_SOCKOPT_SO_PASSCRED
#define NXT_HAVE_SOCKOPT_SO_PASSCRED  1
#endif


#ifndef NXT_HAVE_UCRED
#define NXT_HAVE_UCRED  1
#endif


#ifndef NXT_HAVE_FIONBIO
#define NXT_HAVE_FIONBIO  1
#endif


#ifndef NXT_HAVE_SOCK_NONBLOCK
#define NXT_HAVE_SOCK_NONBLOCK  1
#endif


#ifndef NXT_HAVE_ACCEPT4
#define NXT_HAVE_ACCEPT4  1
#endif


#ifndef NXT_HAVE_LINUX_SENDFILE
#define NXT_HAVE_LINUX_SENDFILE  1
#endif


#ifndef NXT_HAVE_POSIX_FADVISE
#define NXT_HAVE_POSIX_FADVISE  1
#endif


#ifndef NXT_HAVE_OPENAT2
#define NXT_HAVE_OPENAT2  1
#endif


#ifndef NXT_HAVE_GETRANDOM
#define NXT_HAVE_GETRANDOM  1
#endif


#ifndef NXT_HAVE_UCONTEXT
#define NXT_HAVE_UCONTEXT  1
#endif


#ifndef NXT_HAVE_DLOPEN
#define NXT_HAVE_DLOPEN  1
#endif


#ifndef NXT_HAVE_POSIX_SPAWN
#define NXT_HAVE_POSIX_SPAWN  1
#endif


#ifndef NXT_HAVE_GETGROUPLIST
#define NXT_HAVE_GETGROUPLIST  1
#endif


#ifndef NXT_LINUX
#define NXT_LINUX  1
#endif


#ifndef NXT_HAVE_OPENSSL
#define NXT_HAVE_OPENSSL  1
#endif


#ifndef NXT_HAVE_OPENSSL_VERSION
#define NXT_HAVE_OPENSSL_VERSION  "OpenSSL 3.0.2 15 Mar 2022"
#endif


#ifndef NXT_HAVE_OPENSSL_CONF_CMD
#define NXT_HAVE_OPENSSL_CONF_CMD  1
#endif


#ifndef NXT_HAVE_OPENSSL_TLSEXT
#define NXT_HAVE_OPENSSL_TLSEXT  1
#endif


#ifndef NXT_HAVE_PCRE2
#define NXT_HAVE_PCRE2  1
#endif


#ifndef NXT_HAVE_REGEX
#define NXT_HAVE_REGEX  1
#endif


#ifndef NXT_HAVE_CLONE
#define NXT_HAVE_CLONE  1
#endif


#ifndef NXT_HAVE_CLONE_NEWUSER
#define NXT_HAVE_CLONE_NEWUSER  1
#endif


#ifndef NXT_HAVE_CLONE_NEWNS
#define NXT_HAVE_CLONE_NEWNS  1
#endif


#ifndef NXT_HAVE_CLONE_NEWPID
#define NXT_HAVE_CLONE_NEWPID  1
#endif


#ifndef NXT_HAVE_CLONE_NEWNET
#define NXT_HAVE_CLONE_NEWNET  1
#endif


#ifndef NXT_HAVE_CLONE_NEWUTS
#define NXT_HAVE_CLONE_NEWUTS  1
#endif


#ifndef NXT_HAVE_CLONE_NEWCGROUP
#define NXT_HAVE_CLONE_NEWCGROUP  1
#endif


#ifndef NXT_HAVE_PIVOT_ROOT
#define NXT_HAVE_PIVOT_ROOT  1
#endif


#ifndef NXT_HAVE_PR_SET_NO_NEW_PRIVS0
#define NXT_HAVE_PR_SET_NO_NEW_PRIVS0  1
#endif


#ifndef NXT_HAVE_LINUX_MOUNT
#define NXT_HAVE_LINUX_MOUNT  1
#endif


#ifndef NXT_HAVE_LINUX_UMOUNT2
#define NXT_HAVE_LINUX_UMOUNT2  1
#endif


#ifndef NXT_HAVE_ISOLATION_ROOTFS
#define NXT_HAVE_ISOLATION_ROOTFS  1
#endif


#ifndef NXT_HAVE_LINUX_CAPABILITY
#define NXT_HAVE_LINUX_CAPABILITY  1
#endif


#ifndef NXT_HAVE_LITTLE_ENDIAN
#define NXT_HAVE_LITTLE_ENDIAN  1
#endif


#ifndef NXT_HAVE_NONALIGNED
#define NXT_HAVE_NONALIGNED  1
#endif


#ifndef NXT_DEBUG
#define NXT_DEBUG  0
#endif

#define NXT_SHM_PREFIX  "/"


#ifndef NXT_TLS
#define NXT_TLS  1
#endif


#ifndef NXT_TESTS
#define NXT_TESTS  1
#endif

