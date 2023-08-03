/*
** 2001 September 15
**
** The author disclaims copyright to this source code.  In place of
** a legal notice, here is a blessing:
**
**    May you do good and not evil.
**    May you find forgiveness for yourself and forgive others.
**    May you share freely, never taking more than you give.
**
*************************************************************************
** Internal interface definitions for SQLite.
**
*/
#ifndef SQLITEINT_H
#define SQLITEINT_H

/* Special Comments:
**
** Some comments have special meaning to the tools that measure test
** coverage:
**
**    NO_TEST                     - The branches on this line are not
**                                  measured by branch coverage.  This is
**                                  used on lines of code that actually
**                                  implement parts of coverage testing.
**
**    OPTIMIZATION-IF-TRUE        - This branch is allowed to alway be false
**                                  and the correct answer is still obtained,
**                                  though perhaps more slowly.
**
**    OPTIMIZATION-IF-FALSE       - This branch is allowed to alway be true
**                                  and the correct answer is still obtained,
**                                  though perhaps more slowly.
**
**    PREVENTS-HARMLESS-OVERREAD  - This branch prevents a buffer overread
**                                  that would be harmless and undetectable
**                                  if it did occur.  
**
** In all cases, the special comment must be enclosed in the usual
** slash-asterisk...asterisk-slash comment marks, with no spaces between the 
** asterisks and the comment text.
*/

/*
** Make sure the Tcl calling convention macro is defined.  This macro is
** only used by test code and Tcl integration code.
*/
#ifndef SQLITE_TCLAPI
#  define SQLITE_TCLAPI
#endif

/*
** Include the header file used to customize the compiler options for MSVC.
** This should be done first so that it can successfully prevent spurious
** compiler warnings due to subsequent content in this file and other files
** that are included by this file.
*/
#include "msvc.h"

/*
** Special setup for VxWorks
*/
#include "vxworks.h"

/*
** These #defines should enable >2GB file support on POSIX if the
** underlying operating system supports it.  If the OS lacks
** large file support, or if the OS is windows, these should be no-ops.
**
** Ticket #2739:  The _LARGEFILE_SOURCE macro must appear before any
** system #includes.  Hence, this block of code must be the very first
** code in all source files.
**
** Large file support can be disabled using the -DSQLITE_DISABLE_LFS switch
** on the compiler command line.  This is necessary if you are compiling
** on a recent machine (ex: Red Hat 7.2) but you want your code to work
** on an older machine (ex: Red Hat 6.0).  If you compile on Red Hat 7.2
** without this option, LFS is enable.  But LFS does not exist in the kernel
** in Red Hat 6.0, so the code won't work.  Hence, for maximum binary
** portability you should omit LFS.
**
** The previous paragraph was written in 2005.  (This paragraph is written
** on 2008-11-28.) These days, all Linux kernels support large files, so
** you should probably leave LFS enabled.  But some embedded platforms might
** lack LFS in which case the SQLITE_DISABLE_LFS macro might still be useful.
**
** Similar is true for Mac OS X.  LFS is only supported on Mac OS X 9 and later.
*/
#ifndef SQLITE_DISABLE_LFS
# define _LARGE_FILE       1
# ifndef _FILE_OFFSET_BITS
#   define _FILE_OFFSET_BITS 64
# endif
# define _LARGEFILE_SOURCE 1
#endif

/* The GCC_VERSION and MSVC_VERSION macros are used to
** conditionally include optimizations for each of these compilers.  A
** value of 0 means that compiler is not being used.  The
** SQLITE_DISABLE_INTRINSIC macro means do not use any compiler-specific
** optimizations, and hence set all compiler macros to 0
**
** There was once also a CLANG_VERSION macro.  However, we learn that the
** version numbers in clang are for "marketing" only and are inconsistent
** and unreliable.  Fortunately, all versions of clang also recognize the
** gcc version numbers and have reasonable settings for gcc version numbers,
** so the GCC_VERSION macro will be set to a correct non-zero value even
** when compiling with clang.
*/
#if defined(__GNUC__) && !defined(SQLITE_DISABLE_INTRINSIC)
# define GCC_VERSION (__GNUC__*1000000+__GNUC_MINOR__*1000+__GNUC_PATCHLEVEL__)
#else
# define GCC_VERSION 0
#endif
#if defined(_MSC_VER) && !defined(SQLITE_DISABLE_INTRINSIC)
# define MSVC_VERSION _MSC_VER
#else
# define MSVC_VERSION 0
#endif

/*
** Some C99 functions in "math.h" are only present for MSVC when its version
** is associated with Visual Studio 2013 or higher.
*/
#ifndef SQLITE_HAVE_C99_MATH_FUNCS
# if MSVC_VERSION==0 || MSVC_VERSION>=1800
#  define SQLITE_HAVE_C99_MATH_FUNCS (1)
# else
#  define SQLITE_HAVE_C99_MATH_FUNCS (0)
# endif
#endif

/* Needed for various definitions... */
#if defined(__GNUC__) && !defined(_GNU_SOURCE)
# define _GNU_SOURCE
#endif

#if defined(__OpenBSD__) && !defined(_BSD_SOURCE)
# define _BSD_SOURCE
#endif

/*
** Macro to disable warnings about missing "break" at the end of a "case".
*/
#if GCC_VERSION>=7000000
# define deliberate_fall_through __attribute__((fallthrough));
#else
# define deliberate_fall_through
#endif

/*
** For MinGW, check to see if we can include the header file containing its
** version information, among other things.  Normally, this internal MinGW
** header file would [only] be included automatically by other MinGW header
** files; however, the contained version information is now required by this
** header file to work around binary compatibility issues (see below) and
** this is the only known way to reliably obtain it.  This entire #if block
** would be completely unnecessary if there was any other way of detecting
** MinGW via their preprocessor (e.g. if they customized their GCC to define
** some MinGW-specific macros).  When compiling for MinGW, either the
** _HAVE_MINGW_H or _HAVE__MINGW_H (note the extra underscore) macro must be
** defined; otherwise, detection of conditions specific to MinGW will be
** disabled.
*/
#if defined(_HAVE_MINGW_H)
# include "mingw.h"
#elif defined(_HAVE__MINGW_H)
# include "_mingw.h"
#endif

/*
** For MinGW version 4.x (and higher), check to see if the _USE_32BIT_TIME_T
** define is required to maintain binary compatibility with the MSVC runtime
** library in use (e.g. for Windows XP).
*/
#if !defined(_USE_32BIT_TIME_T) && !defined(_USE_64BIT_TIME_T) && \
    defined(_WIN32) && !defined(_WIN64) && \
    defined(__MINGW_MAJOR_VERSION) && __MINGW_MAJOR_VERSION >= 4 && \
    defined(__MSVCRT__)
# define _USE_32BIT_TIME_T
#endif

/* Optionally #include a user-defined header, whereby compilation options
** may be set prior to where they take effect, but after platform setup. 
** If SQLITE_CUSTOM_INCLUDE=? is defined, its value names the #include
** file.
*/
#ifdef SQLITE_CUSTOM_INCLUDE
# define INC_STRINGIFY_(f) #f
# define INC_STRINGIFY(f) INC_STRINGIFY_(f)
# include INC_STRINGIFY(SQLITE_CUSTOM_INCLUDE)
#endif

/* The public SQLite interface.  The _FILE_OFFSET_BITS macro must appear
** first in QNX.  Also, the _USE_32BIT_TIME_T macro must appear first for
** MinGW.
*/
#include "sqlite3.h"

/*
** Reuse the STATIC_LRU for mutex access to sqlite3_temp_directory.
*/
#define SQLITE_MUTEX_STATIC_TEMPDIR SQLITE_MUTEX_STATIC_VFS1

/*
** Include the configuration header output by 'configure' if we're using the
** autoconf-based build
*/
#if defined(_HAVE_SQLITE_CONFIG_H) && !defined(SQLITECONFIG_H)
#include "sqlite_cfg.h"
#define SQLITECONFIG_H 1
#endif

#include "sqliteLimit.h"

/* Disable nuisance warnings on Borland compilers */
#if defined(__BORLANDC__)
#pragma warn -rch /* unreachable code */
#pragma warn -ccc /* Condition is always true or false */
#pragma warn -aus /* Assigned value is never used */
#pragma warn -csu /* Comparing signed and unsigned */
#pragma warn -spa /* Suspicious pointer arithmetic */
#endif

/*
** A few places in the code require atomic load/store of aligned
** integer values.
*/
#ifndef __has_extension
# define __has_extension(x) 0     /* compatibility with non-clang compilers */
#endif
#if GCC_VERSION>=4007000 || __has_extension(c_atomic) 
# define SQLITE_ATOMIC_INTRINSICS 1
# define AtomicLoad(PTR)       __atomic_load_n((PTR),__ATOMIC_RELAXED)
# define AtomicStore(PTR,VAL)  __atomic_store_n((PTR),(VAL),__ATOMIC_RELAXED)
#else
# define SQLITE_ATOMIC_INTRINSICS 0
# define AtomicLoad(PTR)       (*(PTR))
# define AtomicStore(PTR,VAL)  (*(PTR) = (VAL))
#endif

/*
** Include standard header files as necessary
*/
#ifdef HAVE_STDINT_H
#include <stdint.h>
#endif
#ifdef HAVE_INTTYPES_H
#include <inttypes.h>
#endif

/*
** The following macros are used to cast pointers to integers and
** integers to pointers.  The way you do this varies from one compiler
** to the next, so we have developed the following set of #if statements
** to generate appropriate macros for a wide range of compilers.
**
** The correct "ANSI" way to do this is to use the intptr_t type.
** Unfortunately, that typedef is not available on all compilers, or
** if it is available, it requires an #include of specific headers
** that vary from one machine to the next.
**
** Ticket #3860:  The llvm-gcc-4.2 compiler from Apple chokes on
** the ((void*)&((char*)0)[X]) construct.  But MSVC chokes on ((void*)(X)).
** So we have to define the macros in different ways depending on the
** compiler.
*/
#if defined(HAVE_STDINT_H)   /* Use this case if we have ANSI headers */
# define SQLITE_INT_TO_PTR(X)  ((void*)(intptr_t)(X))
# define SQLITE_PTR_TO_INT(X)  ((int)(intptr_t)(X))
#elif defined(__PTRDIFF_TYPE__)  /* This case should work for GCC */
# define SQLITE_INT_TO_PTR(X)  ((void*)(__PTRDIFF_TYPE__)(X))
# define SQLITE_PTR_TO_INT(X)  ((int)(__PTRDIFF_TYPE__)(X))
#elif !defined(__GNUC__)       /* Works for compilers other than LLVM */
# define SQLITE_INT_TO_PTR(X)  ((void*)&((char*)0)[X])
# define SQLITE_PTR_TO_INT(X)  ((int)(((char*)X)-(char*)0))
#else                          /* Generates a warning - but it always works */
# define SQLITE_INT_TO_PTR(X)  ((void*)(X))
# define SQLITE_PTR_TO_INT(X)  ((int)(X))
#endif

/*
** A macro to hint to the compiler that a function should not be
** inlined.
*/
#if defined(__GNUC__)
#  define SQLITE_NOINLINE  __attribute__((noinline))
#elif defined(_MSC_VER) && _MSC_VER>=1310
#  define SQLITE_NOINLINE  __declspec(noinline)
#else
#  define SQLITE_NOINLINE
#endif

/*
** Make sure that the compiler intrinsics we desire are enabled when
** compiling with an appropriate version of MSVC unless prevented by
** the SQLITE_DISABLE_INTRINSIC define.
*/
#if !defined(SQLITE_DISABLE_INTRINSIC)
#  if defined(_MSC_VER) && _MSC_VER>=1400
#    if !defined(_WIN32_WCE)
#      include <intrin.h>
#      pragma intrinsic(_byteswap_ushort)
#      pragma intrinsic(_byteswap_ulong)
#      pragma intrinsic(_byteswap_uint64)
#      pragma intrinsic(_ReadWriteBarrier)
#    else
#      include <cmnintrin.h>
#    endif
#  endif
#endif

/*
** The SQLITE_THREADSAFE macro must be defined as 0, 1, or 2.
** 0 means mutexes are permanently disable and the library is never
** threadsafe.  1 means the library is serialized which is the highest
** level of threadsafety.  2 means the library is multithreaded - multiple
** threads can use SQLite as long as no two threads try to use the same
** database connection at the same time.
**
** Older versions of SQLite used an optional THREADSAFE macro.
** We support that for legacy.
**
** To ensure that the correct value of "THREADSAFE" is reported when querying
** for compile-time options at runtime (e.g. "PRAGMA compile_options"), this
** logic is partially replicated in ctime.c. If it is updated here, it should
** also be updated there.
*/
#if !defined(SQLITE_THREADSAFE)
# if defined(THREADSAFE)
#   define SQLITE_THREADSAFE THREADSAFE
# else
#   define SQLITE_THREADSAFE 1 /* IMP: R-07272-22309 */
# endif
#endif

/*
** Powersafe overwrite is on by default.  But can be turned off using
** the -DSQLITE_POWERSAFE_OVERWRITE=0 command-line option.
*/
#ifndef SQLITE_POWERSAFE_OVERWRITE
# define SQLITE_POWERSAFE_OVERWRITE 1
#endif

/*
** EVIDENCE-OF: R-25715-37072 Memory allocation statistics are enabled by
** default unless SQLite is compiled with SQLITE_DEFAULT_MEMSTATUS=0 in
** which case memory allocation statistics are disabled by default.
*/
#if !defined(SQLITE_DEFAULT_MEMSTATUS)
# define SQLITE_DEFAULT_MEMSTATUS 1
#endif

/*
** Exactly one of the following macros must be defined in order to
** specify which memory allocation subsystem to use.
**
**     SQLITE_SYSTEM_MALLOC          // Use normal system malloc()
**     SQLITE_WIN32_MALLOC           // Use Win32 native heap API
**     SQLITE_ZERO_MALLOC            // Use a stub allocator that always fails
**     SQLITE_MEMDEBUG               // Debugging version of system malloc()
**
** On Windows, if the SQLITE_WIN32_MALLOC_VALIDATE macro is defined and the
** assert() macro is enabled, each call into the Win32 native heap subsystem
** will cause HeapValidate to be called.  If heap validation should fail, an
** assertion will be triggered.
**
** If none of the above are defined, then set SQLITE_SYSTEM_MALLOC as
** the default.
*/
#if defined(SQLITE_SYSTEM_MALLOC) \
  + defined(SQLITE_WIN32_MALLOC) \
  + defined(SQLITE_ZERO_MALLOC) \
  + defined(SQLITE_MEMDEBUG)>1
# error "Two or more of the following compile-time configuration options\
 are defined but at most one is allowed:\
 SQLITE_SYSTEM_MALLOC, SQLITE_WIN32_MALLOC, SQLITE_MEMDEBUG,\
 SQLITE_ZERO_MALLOC"
#endif
#if defined(SQLITE_SYSTEM_MALLOC) \
  + defined(SQLITE_WIN32_MALLOC) \
  + defined(SQLITE_ZERO_MALLOC) \
  + defined(SQLITE_MEMDEBUG)==0
# define SQLITE_SYSTEM_MALLOC 1
#endif

/*
** If SQLITE_MALLOC_SOFT_LIMIT is not zero, then try to keep the
** sizes of memory allocations below this value where possible.
*/
#if !defined(SQLITE_MALLOC_SOFT_LIMIT)
# define SQLITE_MALLOC_SOFT_LIMIT 1024
#endif

/*
** We need to define _XOPEN_SOURCE as follows in order to enable
** recursive mutexes on most Unix systems and fchmod() on OpenBSD.
** But _XOPEN_SOURCE define causes problems for Mac OS X, so omit
** it.
*/
#if !defined(_XOPEN_SOURCE) && !defined(__DARWIN__) && !defined(__APPLE__)
#  define _XOPEN_SOURCE 600
#endif

/*
** NDEBUG and SQLITE_DEBUG are opposites.  It should always be true that
** defined(NDEBUG)==!defined(SQLITE_DEBUG).  If this is not currently true,
** make it true by defining or undefining NDEBUG.
**
** Setting NDEBUG makes the code smaller and faster by disabling the
** assert() statements in the code.  So we want the default action
** to be for NDEBUG to be set and NDEBUG to be undefined only if SQLITE_DEBUG
** is set.  Thus NDEBUG becomes an opt-in rather than an opt-out
** feature.
*/
#if !defined(NDEBUG) && !defined(SQLITE_DEBUG)
# define NDEBUG 1
#endif
#if defined(NDEBUG) && defined(SQLITE_DEBUG)
# undef NDEBUG
#endif

/*
** Enable SQLITE_ENABLE_EXPLAIN_COMMENTS if SQLITE_DEBUG is turned on.
*/
#if !defined(SQLITE_ENABLE_EXPLAIN_COMMENTS) && defined(SQLITE_DEBUG)
# define SQLITE_ENABLE_EXPLAIN_COMMENTS 1
#endif

/*
** The testcase() macro is used to aid in coverage testing.  When
** doing coverage testing, the condition inside the argument to
** testcase() must be evaluated both true and false in order to
** get full branch coverage.  The testcase() macro is inserted
** to help ensure adequate test coverage in places where simple
** condition/decision coverage is inadequate.  For example, testcase()
** can be used to make sure boundary values are tested.  For
** bitmask tests, testcase() can be used to make sure each bit
** is significant and used at least once.  On switch statements
** where multiple cases go to the same block of code, testcase()
** can insure that all cases are evaluated.
*/
#if defined(SQLITE_COVERAGE_TEST) || defined(SQLITE_DEBUG)
# ifndef SQLITE_AMALGAMATION
    extern unsigned int sqlite3CoverageCounter;
# endif
# define testcase(X)  if( X ){ sqlite3CoverageCounter += (unsigned)__LINE__; }
#else
# define testcase(X)
#endif

/*
** The TESTONLY macro is used to enclose variable declarations or
** other bits of code that are needed to support the arguments
** within testcase() and assert() macros.
*/
#if !defined(NDEBUG) || defined(SQLITE_COVERAGE_TEST)
# define TESTONLY(X)  X
#else
# define TESTONLY(X)
#endif

/*
** Sometimes we need a small amount of code such as a variable initialization
** to setup for a later assert() statement.  We do not want this code to
** appear when assert() is disabled.  The following macro is therefore
** used to contain that setup code.  The "VVA" acronym stands for
** "Verification, Validation, and Accreditation".  In other words, the
** code within VVA_ONLY() will only run during verification processes.
*/
#ifndef NDEBUG
# define VVA_ONLY(X)  X
#else
# define VVA_ONLY(X)
#endif

/*
** Disable ALWAYS() and NEVER() (make them pass-throughs) for coverage
** and mutation testing
*/
#if defined(SQLITE_COVERAGE_TEST) || defined(SQLITE_MUTATION_TEST)
# define SQLITE_OMIT_AUXILIARY_SAFETY_CHECKS  1
#endif

/*
** The ALWAYS and NEVER macros surround boolean expressions which
** are intended to always be true or false, respectively.  Such
** expressions could be omitted from the code completely.  But they
** are included in a few cases in order to enhance the resilience
** of SQLite to unexpected behavior - to make the code "self-healing"
** or "ductile" rather than being "brittle" and crashing at the first
** hint of unplanned behavior.
**
** In other words, ALWAYS and NEVER are added for defensive code.
**
** When doing coverage testing ALWAYS and NEVER are hard-coded to
** be true and false so that the unreachable code they specify will
** not be counted as untested code.
*/
#if defined(SQLITE_OMIT_AUXILIARY_SAFETY_CHECKS)
# define ALWAYS(X)      (1)
# define NEVER(X)       (0)
#elif !defined(NDEBUG)
# define ALWAYS(X)      ((X)?1:(assert(0),0))
# define NEVER(X)       ((X)?(assert(0),1):0)
#else
# define ALWAYS(X)      (X)
# define NEVER(X)       (X)
#endif

/*
** Some conditionals are optimizations only.  In other words, if the
** conditionals are replaced with a constant 1 (true) or 0 (false) then
** the correct answer is still obtained, though perhaps not as quickly.
**
** The following macros mark these optimizations conditionals.
*/
#if defined(SQLITE_MUTATION_TEST)
# define OK_IF_ALWAYS_TRUE(X)  (1)
# define OK_IF_ALWAYS_FALSE(X) (0)
#else
# define OK_IF_ALWAYS_TRUE(X)  (X)
# define OK_IF_ALWAYS_FALSE(X) (X)
#endif

/*
** Some malloc failures are only possible if SQLITE_TEST_REALLOC_STRESS is
** defined.  We need to defend against those failures when testing with
** SQLITE_TEST_REALLOC_STRESS, but we don't want the unreachable branches
** during a normal build.  The following macro can be used to disable tests
** that are always false except when SQLITE_TEST_REALLOC_STRESS is set.
*/
#if defined(SQLITE_TEST_REALLOC_STRESS)
# define ONLY_IF_REALLOC_STRESS(X)  (X)
#elif !defined(NDEBUG)
# define ONLY_IF_REALLOC_STRESS(X)  ((X)?(assert(0),1):0)
#else
# define ONLY_IF_REALLOC_STRESS(X)  (0)
#endif

/*
** Declarations used for tracing the operating system interfaces.
*/
#if defined(SQLITE_FORCE_OS_TRACE) || defined(SQLITE_TEST) || \
    (defined(SQLITE_DEBUG) && SQLITE_OS_WIN)
  extern int sqlite3OSTrace;
# define OSTRACE(X)          if( sqlite3OSTrace ) sqlite3DebugPrintf X
# define SQLITE_HAVE_OS_TRACE
#else
# define OSTRACE(X)
# undef  SQLITE_HAVE_OS_TRACE
#endif

/*
** Is the sqlite3ErrName() function needed in the build?  Currently,
** it is needed by "mutex_w32.c" (when debugging), "os_win.c" (when
** OSTRACE is enabled), and by several "test*.c" files (which are
** compiled using SQLITE_TEST).
*/
#if defined(SQLITE_HAVE_OS_TRACE) || defined(SQLITE_TEST) || \
    (defined(SQLITE_DEBUG) && SQLITE_OS_WIN)
# define SQLITE_NEED_ERR_NAME
#else
# undef  SQLITE_NEED_ERR_NAME
#endif

/*
** SQLITE_ENABLE_EXPLAIN_COMMENTS is incompatible with SQLITE_OMIT_EXPLAIN
*/
#ifdef SQLITE_OMIT_EXPLAIN
# undef SQLITE_ENABLE_EXPLAIN_COMMENTS
#endif

/*
** SQLITE_OMIT_VIRTUALTABLE implies SQLITE_OMIT_ALTERTABLE
*/
#if defined(SQLITE_OMIT_VIRTUALTABLE) && !defined(SQLITE_OMIT_ALTERTABLE)
# define SQLITE_OMIT_ALTERTABLE
#endif

/*
** Return true (non-zero) if the input is an integer that is too large
** to fit in 32-bits.  This macro is used inside of various testcase()
** macros to verify that we have tested SQLite for large-file support.
*/
#define IS_BIG_INT(X)  (((X)&~(i64)0xffffffff)!=0)

/*
** The macro unlikely() is a hint that surrounds a boolean
** expression that is usually false.  Macro likely() surrounds
** a boolean expression that is usually true.  These hints could,
** in theory, be used by the compiler to generate better code, but
** currently they are just comments for human readers.
*/
#define likely(X)    (X)
#define unlikely(X)  (X)

#include "sqlite3_rs.h"
#include "parse.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <stddef.h>

/*
** If compiling for a processor that lacks floating point support,
** substitute integer for floating-point
*/
#ifdef SQLITE_OMIT_FLOATING_POINT
# define double sqlite_int64
# define float sqlite_int64
# define LONGDOUBLE_TYPE sqlite_int64
# ifndef SQLITE_BIG_DBL
#   define SQLITE_BIG_DBL (((sqlite3_int64)1)<<50)
# endif
# define SQLITE_OMIT_DATETIME_FUNCS 1
# define SQLITE_OMIT_TRACE 1
# undef SQLITE_MIXED_ENDIAN_64BIT_FLOAT
# undef SQLITE_HAVE_ISNAN
#endif
#ifndef SQLITE_BIG_DBL
# define SQLITE_BIG_DBL (1e99)
#endif

/*
** OMIT_TEMPDB is set to 1 if SQLITE_OMIT_TEMPDB is defined, or 0
** afterward. Having this macro allows us to cause the C compiler
** to omit code used by TEMP tables without messy #ifndef statements.
*/
#ifdef SQLITE_OMIT_TEMPDB
#define OMIT_TEMPDB 1
#else
#define OMIT_TEMPDB 0
#endif

/*
** The "file format" number is an integer that is incremented whenever
** the VDBE-level file format changes.  The following macros define the
** the default file format for new databases and the maximum file format
** that the library can read.
*/
#define SQLITE_MAX_FILE_FORMAT 4
#ifndef SQLITE_DEFAULT_FILE_FORMAT
# define SQLITE_DEFAULT_FILE_FORMAT 4
#endif

/*
** Determine whether triggers are recursive by default.  This can be
** changed at run-time using a pragma.
*/
#ifndef SQLITE_DEFAULT_RECURSIVE_TRIGGERS
# define SQLITE_DEFAULT_RECURSIVE_TRIGGERS 0
#endif

/*
** Provide a default value for SQLITE_TEMP_STORE in case it is not specified
** on the command-line
*/
#ifndef SQLITE_TEMP_STORE
# define SQLITE_TEMP_STORE 1
#endif

/*
** If no value has been provided for SQLITE_MAX_WORKER_THREADS, or if
** SQLITE_TEMP_STORE is set to 3 (never use temporary files), set it
** to zero.
*/
#if SQLITE_TEMP_STORE==3 || SQLITE_THREADSAFE==0
# undef SQLITE_MAX_WORKER_THREADS
# define SQLITE_MAX_WORKER_THREADS 0
#endif
#ifndef SQLITE_MAX_WORKER_THREADS
# define SQLITE_MAX_WORKER_THREADS 8
#endif
#ifndef SQLITE_DEFAULT_WORKER_THREADS
# define SQLITE_DEFAULT_WORKER_THREADS 0
#endif
#if SQLITE_DEFAULT_WORKER_THREADS>SQLITE_MAX_WORKER_THREADS
# undef SQLITE_MAX_WORKER_THREADS
# define SQLITE_MAX_WORKER_THREADS SQLITE_DEFAULT_WORKER_THREADS
#endif

/*
** The default initial allocation for the pagecache when using separate
** pagecaches for each database connection.  A positive number is the
** number of pages.  A negative number N translations means that a buffer
** of -1024*N bytes is allocated and used for as many pages as it will hold.
**
** The default value of "20" was chosen to minimize the run-time of the
** speedtest1 test program with options: --shrink-memory --reprepare
*/
#ifndef SQLITE_DEFAULT_PCACHE_INITSZ
# define SQLITE_DEFAULT_PCACHE_INITSZ 20
#endif

/*
** Default value for the SQLITE_CONFIG_SORTERREF_SIZE option.
*/
#ifndef SQLITE_DEFAULT_SORTERREF_SIZE
# define SQLITE_DEFAULT_SORTERREF_SIZE 0x7fffffff
#endif

/*
** The compile-time options SQLITE_MMAP_READWRITE and 
** SQLITE_ENABLE_BATCH_ATOMIC_WRITE are not compatible with one another.
** You must choose one or the other (or neither) but not both.
*/
#if defined(SQLITE_MMAP_READWRITE) && defined(SQLITE_ENABLE_BATCH_ATOMIC_WRITE)
#error Cannot use both SQLITE_MMAP_READWRITE and SQLITE_ENABLE_BATCH_ATOMIC_WRITE
#endif

/*
** GCC does not define the offsetof() macro so we'll have to do it
** ourselves.
*/
#ifndef offsetof
#define offsetof(STRUCTURE,FIELD) ((int)((char*)&((STRUCTURE*)0)->FIELD))
#endif

/*
** Macros to compute minimum and maximum of two numbers.
*/
#ifndef MIN
# define MIN(A,B) ((A)<(B)?(A):(B))
#endif
#ifndef MAX
# define MAX(A,B) ((A)>(B)?(A):(B))
#endif

/*
** Swap two objects of type TYPE.
*/
#define SWAP(TYPE,A,B) {TYPE t=A; A=B; B=t;}

# define SQLITE_ASCII 1

/*
** Integers of known sizes.  These typedefs might change for architectures
** where the sizes very.  Preprocessor macros are available so that the
** types can be conveniently redefined at compile-type.  Like this:
**
**         cc '-DUINTPTR_TYPE=long long int' ...
*/
#ifndef UINT32_TYPE
# ifdef HAVE_UINT32_T
#  define UINT32_TYPE uint32_t
# else
#  define UINT32_TYPE unsigned int
# endif
#endif
#ifndef UINT16_TYPE
# ifdef HAVE_UINT16_T
#  define UINT16_TYPE uint16_t
# else
#  define UINT16_TYPE unsigned short int
# endif
#endif
#ifndef INT16_TYPE
# ifdef HAVE_INT16_T
#  define INT16_TYPE int16_t
# else
#  define INT16_TYPE short int
# endif
#endif
#ifndef UINT8_TYPE
# ifdef HAVE_UINT8_T
#  define UINT8_TYPE uint8_t
# else
#  define UINT8_TYPE unsigned char
# endif
#endif
#ifndef INT8_TYPE
# ifdef HAVE_INT8_T
#  define INT8_TYPE int8_t
# else
#  define INT8_TYPE signed char
# endif
#endif
#ifndef LONGDOUBLE_TYPE
# define LONGDOUBLE_TYPE long double
#endif
typedef sqlite_int64 i64;          /* 8-byte signed integer */
typedef sqlite_uint64 u64;         /* 8-byte unsigned integer */
typedef UINT32_TYPE u32;           /* 4-byte unsigned integer */
typedef UINT16_TYPE u16;           /* 2-byte unsigned integer */
typedef INT16_TYPE i16;            /* 2-byte signed integer */
typedef UINT8_TYPE u8;             /* 1-byte unsigned integer */
typedef INT8_TYPE i8;              /* 1-byte signed integer */

/*
** Set the SQLITE_PTRSIZE macro to the number of bytes in a pointer
*/
#ifndef SQLITE_PTRSIZE
# if defined(__SIZEOF_POINTER__)
#   define SQLITE_PTRSIZE __SIZEOF_POINTER__
# elif defined(i386)     || defined(__i386__)   || defined(_M_IX86) ||    \
       defined(_M_ARM)   || defined(__arm__)    || defined(__x86)   ||    \
      (defined(__APPLE__) && defined(__POWERPC__)) ||                     \
      (defined(__TOS_AIX__) && !defined(__64BIT__))
#   define SQLITE_PTRSIZE 4
# else
#   define SQLITE_PTRSIZE 8
# endif
#endif

/* The uptr type is an unsigned integer large enough to hold a pointer
*/
#if defined(HAVE_STDINT_H)
  typedef uintptr_t uptr;
#elif SQLITE_PTRSIZE==4
  typedef u32 uptr;
#else
  typedef u64 uptr;
#endif

/*
** The SQLITE_WITHIN(P,S,E) macro checks to see if pointer P points to
** something between S (inclusive) and E (exclusive).
**
** In other words, S is a buffer and E is a pointer to the first byte after
** the end of buffer S.  This macro returns true if P points to something
** contained within the buffer S.
*/
#define SQLITE_WITHIN(P,S,E) (((uptr)(P)>=(uptr)(S))&&((uptr)(P)<(uptr)(E)))


/*
** Macros to determine whether the machine is big or little endian,
** and whether or not that determination is run-time or compile-time.
**
** For best performance, an attempt is made to guess at the byte-order
** using C-preprocessor macros.  If that is unsuccessful, or if
** -DSQLITE_BYTEORDER=0 is set, then byte-order is determined
** at run-time.
*/
#ifndef SQLITE_BYTEORDER
# if defined(i386)      || defined(__i386__)      || defined(_M_IX86) ||    \
     defined(__x86_64)  || defined(__x86_64__)    || defined(_M_X64)  ||    \
     defined(_M_AMD64)  || defined(_M_ARM)        || defined(__x86)   ||    \
     defined(__ARMEL__) || defined(__AARCH64EL__) || defined(_M_ARM64)
#   define SQLITE_BYTEORDER    1234
# elif defined(sparc)     || defined(__ppc__) || \
       defined(__ARMEB__) || defined(__AARCH64EB__)
#   define SQLITE_BYTEORDER    4321
# else
#   define SQLITE_BYTEORDER 0
# endif
#endif
#if SQLITE_BYTEORDER==4321
# define SQLITE_BIGENDIAN    1
# define SQLITE_LITTLEENDIAN 0
# define SQLITE_UTF16NATIVE  SQLITE_UTF16BE
#elif SQLITE_BYTEORDER==1234
# define SQLITE_BIGENDIAN    0
# define SQLITE_LITTLEENDIAN 1
# define SQLITE_UTF16NATIVE  SQLITE_UTF16LE
#else
# ifdef SQLITE_AMALGAMATION
  const int sqlite3one = 1;
# else
  extern const int sqlite3one;
# endif
# define SQLITE_BIGENDIAN    (*(char *)(&sqlite3one)==0)
# define SQLITE_LITTLEENDIAN (*(char *)(&sqlite3one)==1)
# define SQLITE_UTF16NATIVE  (SQLITE_BIGENDIAN?SQLITE_UTF16BE:SQLITE_UTF16LE)
#endif

/*
** Round up a number to the next larger multiple of 8.  This is used
** to force 8-byte alignment on 64-bit architectures.
**
** ROUND8() always does the rounding, for any argument.
**
** ROUND8P() assumes that the argument is already an integer number of
** pointers in size, and so it is a no-op on systems where the pointer
** size is 8.
*/
#define ROUND8(x)     (((x)+7)&~7)
#if SQLITE_PTRSIZE==8
# define ROUND8P(x)   (x)
#else
# define ROUND8P(x)   (((x)+7)&~7)
#endif

/*
** Round down to the nearest multiple of 8
*/
#define ROUNDDOWN8(x) ((x)&~7)

/*
** Assert that the pointer X is aligned to an 8-byte boundary.  This
** macro is used only within assert() to verify that the code gets
** all alignment restrictions correct.
**
** Except, if SQLITE_4_BYTE_ALIGNED_MALLOC is defined, then the
** underlying malloc() implementation might return us 4-byte aligned
** pointers.  In that case, only verify 4-byte alignment.
*/
#ifdef SQLITE_4_BYTE_ALIGNED_MALLOC
# define EIGHT_BYTE_ALIGNMENT(X)   ((((uptr)(X) - (uptr)0)&3)==0)
#else
# define EIGHT_BYTE_ALIGNMENT(X)   ((((uptr)(X) - (uptr)0)&7)==0)
#endif

/*
** Disable MMAP on platforms where it is known to not work
*/
#if defined(__OpenBSD__) || defined(__QNXNTO__)
# undef SQLITE_MAX_MMAP_SIZE
# define SQLITE_MAX_MMAP_SIZE 0
#endif

/*
** Default maximum size of memory used by memory-mapped I/O in the VFS
*/
#ifdef __APPLE__
# include <TargetConditionals.h>
#endif
#ifndef SQLITE_MAX_MMAP_SIZE
# if defined(__linux__) \
  || defined(_WIN32) \
  || (defined(__APPLE__) && defined(__MACH__)) \
  || defined(__sun) \
  || defined(__FreeBSD__) \
  || defined(__DragonFly__)
#   define SQLITE_MAX_MMAP_SIZE 0x7fff0000  /* 2147418112 */
# else
#   define SQLITE_MAX_MMAP_SIZE 0
# endif
#endif

/*
** The default MMAP_SIZE is zero on all platforms.  Or, even if a larger
** default MMAP_SIZE is specified at compile-time, make sure that it does
** not exceed the maximum mmap size.
*/
#ifndef SQLITE_DEFAULT_MMAP_SIZE
# define SQLITE_DEFAULT_MMAP_SIZE 0
#endif
#if SQLITE_DEFAULT_MMAP_SIZE>SQLITE_MAX_MMAP_SIZE
# undef SQLITE_DEFAULT_MMAP_SIZE
# define SQLITE_DEFAULT_MMAP_SIZE SQLITE_MAX_MMAP_SIZE
#endif

/*
** TREETRACE_ENABLED will be either 1 or 0 depending on whether or not
** the Abstract Syntax Tree tracing logic is turned on.
*/
#if !defined(SQLITE_AMALGAMATION)
extern u32 sqlite3TreeTrace;
#endif
#if defined(SQLITE_DEBUG) \
    && (defined(SQLITE_TEST) || defined(SQLITE_ENABLE_SELECTTRACE) \
                             || defined(SQLITE_ENABLE_TREETRACE))
# define TREETRACE_ENABLED 1
# define TREETRACE(K,P,S,X)  \
  if(sqlite3TreeTrace&(K))   \
    sqlite3DebugPrintf("%u/%d/%p: ",(S)->selId,(P)->addrExplain,(S)),\
    sqlite3DebugPrintf X
#else
# define TREETRACE(K,P,S,X)
# define TREETRACE_ENABLED 0
#endif

/* TREETRACE flag meanings:
**
**   0x00000001     Beginning and end of SELECT processing
**   0x00000002     WHERE clause processing
**   0x00000004     Query flattener
**   0x00000008     Result-set wildcard expansion
**   0x00000010     Query name resolution
**   0x00000020     Aggregate analysis
**   0x00000040     Window functions
**   0x00000080     Generated column names
**   0x00000100     Move HAVING terms into WHERE
**   0x00000200     Count-of-view optimization
**   0x00000400     Compound SELECT processing
**   0x00000800     Drop superfluous ORDER BY
**   0x00001000     LEFT JOIN simplifies to JOIN
**   0x00002000     Constant propagation
**   0x00004000     Push-down optimization
**   0x00008000     After all FROM-clause analysis
**   0x00010000     Beginning of DELETE/INSERT/UPDATE processing
**   0x00020000     Transform DISTINCT into GROUP BY
**   0x00040000     SELECT tree dump after all code has been generated
*/

/*
** Macros for "wheretrace"
*/
extern u32 sqlite3WhereTrace;
#if defined(SQLITE_DEBUG) \
    && (defined(SQLITE_TEST) || defined(SQLITE_ENABLE_WHERETRACE))
# define WHERETRACE(K,X)  if(sqlite3WhereTrace&(K)) sqlite3DebugPrintf X
# define WHERETRACE_ENABLED 1
#else
# define WHERETRACE(K,X)
#endif

/*
** Bits for the sqlite3WhereTrace mask:
**
** (---any--)   Top-level block structure
** 0x-------F   High-level debug messages
** 0x----FFF-   More detail
** 0xFFFF----   Low-level debug messages
**
** 0x00000001   Code generation
** 0x00000002   Solver
** 0x00000004   Solver costs
** 0x00000008   WhereLoop inserts
**
** 0x00000010   Display sqlite3_index_info xBestIndex calls
** 0x00000020   Range an equality scan metrics
** 0x00000040   IN operator decisions
** 0x00000080   WhereLoop cost adjustements
** 0x00000100
** 0x00000200   Covering index decisions
** 0x00000400   OR optimization
** 0x00000800   Index scanner
** 0x00001000   More details associated with code generation
** 0x00002000
** 0x00004000   Show all WHERE terms at key points
** 0x00008000   Show the full SELECT statement at key places
**
** 0x00010000   Show more detail when printing WHERE terms
** 0x00020000   Show WHERE terms returned from whereScanNext()
*/


/*
** Name of table that holds the database schema.
**
** The PREFERRED names are used whereever possible.  But LEGACY is also
** used for backwards compatibility.
**
**  1.  Queries can use either the PREFERRED or the LEGACY names
**  2.  The sqlite3_set_authorizer() callback uses the LEGACY name
**  3.  The PRAGMA table_list statement uses the PREFERRED name
**
** The LEGACY names are stored in the internal symbol hash table
** in support of (2).  Names are translated using sqlite3PreferredTableName()
** for (3).  The sqlite3FindTable() function takes care of translating
** names for (1).
**
** Note that "sqlite_temp_schema" can also be called "temp.sqlite_schema".
*/
#define LEGACY_SCHEMA_TABLE          "sqlite_master"
#define LEGACY_TEMP_SCHEMA_TABLE     "sqlite_temp_master"
#define PREFERRED_SCHEMA_TABLE       "sqlite_schema"
#define PREFERRED_TEMP_SCHEMA_TABLE  "sqlite_temp_schema"


/*
** The root-page of the schema table.
*/
#define SCHEMA_ROOT    1

/*
** The name of the schema table.  The name is different for TEMP.
*/
#define SCHEMA_TABLE(x) \
    ((!OMIT_TEMPDB)&&(x==1)?LEGACY_TEMP_SCHEMA_TABLE:LEGACY_SCHEMA_TABLE)

/*
** A convenience macro that returns the number of elements in
** an array.
*/
#define ArraySize(X)    ((int)(sizeof(X)/sizeof(X[0])))

/*
** Determine if the argument is a power of two
*/
#define IsPowerOfTwo(X) (((X)&((X)-1))==0)

/*
** The following value as a destructor means to use sqlite3DbFree().
** The sqlite3DbFree() routine requires two parameters instead of the
** one parameter that destructors normally want.  So we have to introduce
** this magic value that the code knows to handle differently.  Any
** pointer will work here as long as it is distinct from SQLITE_STATIC
** and SQLITE_TRANSIENT.
*/
#define SQLITE_DYNAMIC   ((sqlite3_destructor_type)sqlite3OomClear)

/*
** When SQLITE_OMIT_WSD is defined, it means that the target platform does
** not support Writable Static Data (WSD) such as global and static variables.
** All variables must either be on the stack or dynamically allocated from
** the heap.  When WSD is unsupported, the variable declarations scattered
** throughout the SQLite code must become constants instead.  The SQLITE_WSD
** macro is used for this purpose.  And instead of referencing the variable
** directly, we use its constant as a key to lookup the run-time allocated
** buffer that holds real variable.  The constant is also the initializer
** for the run-time allocated buffer.
**
** In the usual case where WSD is supported, the SQLITE_WSD and GLOBAL
** macros become no-ops and have zero performance impact.
*/
#ifdef SQLITE_OMIT_WSD
  #define SQLITE_WSD const
  #define GLOBAL(t,v) (*(t*)sqlite3_wsd_find((void*)&(v), sizeof(v)))
  #define sqlite3GlobalConfig GLOBAL(struct Sqlite3Config, sqlite3Config)
  int sqlite3_wsd_init(int N, int J);
  void *sqlite3_wsd_find(void *K, int L);
#else
  #define SQLITE_WSD
  #define GLOBAL(t,v) v
  #define sqlite3GlobalConfig sqlite3Config
#endif

/*
** The following macros are used to suppress compiler warnings and to
** make it clear to human readers when a function parameter is deliberately
** left unused within the body of a function. This usually happens when
** a function is called via a function pointer. For example the
** implementation of an SQL aggregate step callback may not use the
** parameter indicating the number of arguments passed to the aggregate,
** if it knows that this is enforced elsewhere.
**
** When a function parameter is not used at all within the body of a function,
** it is generally named "NotUsed" or "NotUsed2" to make things even clearer.
** However, these macros may also be used to suppress warnings related to
** parameters that may or may not be used depending on compilation options.
** For example those parameters only used in assert() statements. In these
** cases the parameters are named as per the usual conventions.
*/
#define UNUSED_PARAMETER(x) (void)(x)
#define UNUSED_PARAMETER2(x,y) UNUSED_PARAMETER(x),UNUSED_PARAMETER(y)

/*
** Forward references to structures
*/
typedef struct DbFixer DbFixer;
typedef struct KeyClass KeyClass;
typedef struct PreUpdate PreUpdate;
typedef struct PrintfArguments PrintfArguments;
typedef struct SQLiteThread SQLiteThread;
typedef struct sqlite3_str StrAccum; /* Internal alias for sqlite3_str */
typedef struct TableLock TableLock;
typedef struct TreeView TreeView;
typedef struct Walker Walker;

/* A VList object records a mapping between parameters/variables/wildcards
** in the SQL statement (such as $abc, @pqr, or :xyz) and the integer
** variable number associated with that parameter.  See the format description
** on the sqlite3VListAdd() routine for more information.  A VList is really
** just an array of integers.
*/
typedef int VList;

/*
** Defer sourcing vdbe.h and btree.h until after the "u8" and
** "BusyHandler" typedefs. vdbe.h also requires a few of the opaque
** pointer types (i.e. FuncDef) defined above.
*/
#include "os.h"
#include "pager.h"
#include "btree.h"
#include "vdbe.h"
#include "pcache.h"
#include "mutex.h"

/*
** Default synchronous levels.
**
** Note that (for historcal reasons) the PAGER_SYNCHRONOUS_* macros differ
** from the SQLITE_DEFAULT_SYNCHRONOUS value by 1.
**
**           PAGER_SYNCHRONOUS       DEFAULT_SYNCHRONOUS
**   OFF           1                         0
**   NORMAL        2                         1
**   FULL          3                         2
**   EXTRA         4                         3
**
** The "PRAGMA synchronous" statement also uses the zero-based numbers.
** In other words, the zero-based numbers are used for all external interfaces
** and the one-based values are used internally.
*/
#ifndef SQLITE_DEFAULT_SYNCHRONOUS
# define SQLITE_DEFAULT_SYNCHRONOUS 2
#endif
#ifndef SQLITE_DEFAULT_WAL_SYNCHRONOUS
# define SQLITE_DEFAULT_WAL_SYNCHRONOUS SQLITE_DEFAULT_SYNCHRONOUS
#endif


#define DisableLookaside  db->lookaside.bDisable++;db->lookaside.sz=0
#define EnableLookaside   db->lookaside.bDisable--;\
   db->lookaside.sz=db->lookaside.bDisable?0:db->lookaside.szTrue

/* Size of the smaller allocations in two-size lookside */
#ifdef SQLITE_OMIT_TWOSIZE_LOOKASIDE
#  define LOOKASIDE_SMALL           0
#else
#  define LOOKASIDE_SMALL         128
#endif

#define SQLITE_FUNC_HASH(C,L) (((C)+(L))%SQLITE_FUNC_HASH_SZ)

#ifdef SQLITE_USER_AUTHENTICATION

/* Functions used only by user authorization logic */
int sqlite3UserAuthTable(const char*);
int sqlite3UserAuthCheckLogin(sqlite3*,const char*,u8*);
void sqlite3UserAuthInit(sqlite3*);
void sqlite3CryptFunc(sqlite3_context*,int,sqlite3_value**);

#endif /* SQLITE_USER_AUTHENTICATION */

/*
** typedef for the authorization callback function.
*/
#ifdef SQLITE_USER_AUTHENTICATION
  typedef int (*sqlite3_xauth)(void*,int,const char*,const char*,const char*,
                               const char*, const char*);
#else
  typedef int (*sqlite3_xauth)(void*,int,const char*,const char*,const char*,
                               const char*);
#endif

#ifndef SQLITE_OMIT_DEPRECATED
/* This is an extra SQLITE_TRACE macro that indicates "legacy" tracing
** in the style of sqlite3_trace()
*/
#define SQLITE_TRACE_LEGACY          0x40     /* Use the legacy xTrace */
#define SQLITE_TRACE_XPROFILE        0x80     /* Use the legacy xProfile */
#else
#define SQLITE_TRACE_LEGACY          0
#define SQLITE_TRACE_XPROFILE        0
#endif /* SQLITE_OMIT_DEPRECATED */
#define SQLITE_TRACE_NONLEGACY_MASK  0x0f     /* Normal flags */

/*
** Maximum number of sqlite3.aDb[] entries.  This is the number of attached
** databases plus 2 for "main" and "temp".
*/
#define SQLITE_MAX_DB (SQLITE_MAX_ATTACHED+2)

/*
** A macro to discover the encoding of a database.
*/
#define SCHEMA_ENC(db) ((db)->aDb[0].pSchema->enc)
#define ENC(db)        ((db)->enc)

/*
** A u64 constant where the lower 32 bits are all zeros.  Only the
** upper 32 bits are included in the argument.  Necessary because some
** C-compilers still do not accept LL integer literals.
*/
#define HI(X)  ((u64)(X)<<32)

/*
** Bits of the sqlite3.dbOptFlags field that are used by the
** sqlite3_test_control(SQLITE_TESTCTRL_OPTIMIZATIONS,...) interface to
** selectively disable various optimizations.
*/
#define SQLITE_QueryFlattener 0x00000001 /* Query flattening */
#define SQLITE_WindowFunc     0x00000002 /* Use xInverse for window functions */
#define SQLITE_GroupByOrder   0x00000004 /* GROUPBY cover of ORDERBY */
#define SQLITE_FactorOutConst 0x00000008 /* Constant factoring */
#define SQLITE_DistinctOpt    0x00000010 /* DISTINCT using indexes */
#define SQLITE_CoverIdxScan   0x00000020 /* Covering index scans */
#define SQLITE_OrderByIdxJoin 0x00000040 /* ORDER BY of joins via index */
#define SQLITE_Transitive     0x00000080 /* Transitive constraints */
#define SQLITE_OmitNoopJoin   0x00000100 /* Omit unused tables in joins */
#define SQLITE_CountOfView    0x00000200 /* The count-of-view optimization */
#define SQLITE_CursorHints    0x00000400 /* Add OP_CursorHint opcodes */
#define SQLITE_Stat4          0x00000800 /* Use STAT4 data */
   /* TH3 expects this value  ^^^^^^^^^^ to be 0x0000800. Don't change it */
#define SQLITE_PushDown       0x00001000 /* The push-down optimization */
#define SQLITE_SimplifyJoin   0x00002000 /* Convert LEFT JOIN to JOIN */
#define SQLITE_SkipScan       0x00004000 /* Skip-scans */
#define SQLITE_PropagateConst 0x00008000 /* The constant propagation opt */
#define SQLITE_MinMaxOpt      0x00010000 /* The min/max optimization */
#define SQLITE_SeekScan       0x00020000 /* The OP_SeekScan optimization */
#define SQLITE_OmitOrderBy    0x00040000 /* Omit pointless ORDER BY */
   /* TH3 expects this value  ^^^^^^^^^^ to be 0x40000. Coordinate any change */
#define SQLITE_BloomFilter    0x00080000 /* Use a Bloom filter on searches */
#define SQLITE_BloomPulldown  0x00100000 /* Run Bloom filters early */
#define SQLITE_BalancedMerge  0x00200000 /* Balance multi-way merges */
#define SQLITE_ReleaseReg     0x00400000 /* Use OP_ReleaseReg for testing */
#define SQLITE_FlttnUnionAll  0x00800000 /* Disable the UNION ALL flattener */
   /* TH3 expects this value  ^^^^^^^^^^ See flatten04.test */
#define SQLITE_IndexedExpr    0x01000000 /* Pull exprs from index when able */
#define SQLITE_Coroutines     0x02000000 /* Co-routines for subqueries */
#define SQLITE_NullUnusedCols 0x04000000 /* NULL unused columns in subqueries */
#define SQLITE_AllOpts        0xffffffff /* All optimizations */

/*
** Macros for testing whether or not optimizations are enabled or disabled.
*/
#define OptimizationDisabled(db, mask)  (((db)->dbOptFlags&(mask))!=0)
#define OptimizationEnabled(db, mask)   (((db)->dbOptFlags&(mask))==0)

/*
** Return true if it OK to factor constant expressions into the initialization
** code. The argument is a Parse object for the code generator.
*/
#define ConstFactorOk(P) ((P)->okConstFactor)


/*
** The following three macros, FUNCTION(), LIKEFUNC() and AGGREGATE() are
** used to create the initializers for the FuncDef structures.
**
**   FUNCTION(zName, nArg, iArg, bNC, xFunc)
**     Used to create a scalar function definition of a function zName
**     implemented by C function xFunc that accepts nArg arguments. The
**     value passed as iArg is cast to a (void*) and made available
**     as the user-data (sqlite3_user_data()) for the function. If
**     argument bNC is true, then the SQLITE_FUNC_NEEDCOLL flag is set.
**
**   VFUNCTION(zName, nArg, iArg, bNC, xFunc)
**     Like FUNCTION except it omits the SQLITE_FUNC_CONSTANT flag.
**
**   SFUNCTION(zName, nArg, iArg, bNC, xFunc)
**     Like FUNCTION except it omits the SQLITE_FUNC_CONSTANT flag and
**     adds the SQLITE_DIRECTONLY flag.
**
**   INLINE_FUNC(zName, nArg, iFuncId, mFlags)
**     zName is the name of a function that is implemented by in-line
**     byte code rather than by the usual callbacks. The iFuncId
**     parameter determines the function id.  The mFlags parameter is
**     optional SQLITE_FUNC_ flags for this function.
**
**   TEST_FUNC(zName, nArg, iFuncId, mFlags)
**     zName is the name of a test-only function implemented by in-line
**     byte code rather than by the usual callbacks. The iFuncId
**     parameter determines the function id.  The mFlags parameter is
**     optional SQLITE_FUNC_ flags for this function.
**
**   DFUNCTION(zName, nArg, iArg, bNC, xFunc)
**     Like FUNCTION except it omits the SQLITE_FUNC_CONSTANT flag and
**     adds the SQLITE_FUNC_SLOCHNG flag.  Used for date & time functions
**     and functions like sqlite_version() that can change, but not during
**     a single query.  The iArg is ignored.  The user-data is always set
**     to a NULL pointer.  The bNC parameter is not used.
**
**   MFUNCTION(zName, nArg, xPtr, xFunc)
**     For math-library functions.  xPtr is an arbitrary pointer.
**
**   PURE_DATE(zName, nArg, iArg, bNC, xFunc)
**     Used for "pure" date/time functions, this macro is like DFUNCTION
**     except that it does set the SQLITE_FUNC_CONSTANT flags.  iArg is
**     ignored and the user-data for these functions is set to an 
**     arbitrary non-NULL pointer.  The bNC parameter is not used.
**
**   AGGREGATE(zName, nArg, iArg, bNC, xStep, xFinal)
**     Used to create an aggregate function definition implemented by
**     the C functions xStep and xFinal. The first four parameters
**     are interpreted in the same way as the first 4 parameters to
**     FUNCTION().
**
**   WAGGREGATE(zName, nArg, iArg, xStep, xFinal, xValue, xInverse)
**     Used to create an aggregate function definition implemented by
**     the C functions xStep and xFinal. The first four parameters
**     are interpreted in the same way as the first 4 parameters to
**     FUNCTION().
**
**   LIKEFUNC(zName, nArg, pArg, flags)
**     Used to create a scalar function definition of a function zName
**     that accepts nArg arguments and is implemented by a call to C
**     function likeFunc. Argument pArg is cast to a (void *) and made
**     available as the function user-data (sqlite3_user_data()). The
**     FuncDef.flags variable is set to the value passed as the flags
**     parameter.
*/
#define FUNCTION(zName, nArg, iArg, bNC, xFunc) \
  {nArg, SQLITE_FUNC_BUILTIN|\
   SQLITE_FUNC_CONSTANT|SQLITE_UTF8|(bNC*SQLITE_FUNC_NEEDCOLL), \
   SQLITE_INT_TO_PTR(iArg), 0, xFunc, 0, 0, 0, #zName, {0} }
#define VFUNCTION(zName, nArg, iArg, bNC, xFunc) \
  {nArg, SQLITE_FUNC_BUILTIN|SQLITE_UTF8|(bNC*SQLITE_FUNC_NEEDCOLL), \
   SQLITE_INT_TO_PTR(iArg), 0, xFunc, 0, 0, 0, #zName, {0} }
#define SFUNCTION(zName, nArg, iArg, bNC, xFunc) \
  {nArg, SQLITE_FUNC_BUILTIN|SQLITE_UTF8|SQLITE_DIRECTONLY|SQLITE_FUNC_UNSAFE, \
   SQLITE_INT_TO_PTR(iArg), 0, xFunc, 0, 0, 0, #zName, {0} }
#define MFUNCTION(zName, nArg, xPtr, xFunc) \
  {nArg, SQLITE_FUNC_BUILTIN|SQLITE_FUNC_CONSTANT|SQLITE_UTF8, \
   xPtr, 0, xFunc, 0, 0, 0, #zName, {0} }
#define JFUNCTION(zName, nArg, iArg, xFunc) \
  {nArg, SQLITE_FUNC_BUILTIN|SQLITE_DETERMINISTIC|\
   SQLITE_FUNC_CONSTANT|SQLITE_UTF8, \
   SQLITE_INT_TO_PTR(iArg), 0, xFunc, 0, 0, 0, #zName, {0} }
#define INLINE_FUNC(zName, nArg, iArg, mFlags) \
  {nArg, SQLITE_FUNC_BUILTIN|\
   SQLITE_UTF8|SQLITE_FUNC_INLINE|SQLITE_FUNC_CONSTANT|(mFlags), \
   SQLITE_INT_TO_PTR(iArg), 0, noopFunc, 0, 0, 0, #zName, {0} }
#define TEST_FUNC(zName, nArg, iArg, mFlags) \
  {nArg, SQLITE_FUNC_BUILTIN|\
         SQLITE_UTF8|SQLITE_FUNC_INTERNAL|SQLITE_FUNC_TEST| \
         SQLITE_FUNC_INLINE|SQLITE_FUNC_CONSTANT|(mFlags), \
   SQLITE_INT_TO_PTR(iArg), 0, noopFunc, 0, 0, 0, #zName, {0} }
#define DFUNCTION(zName, nArg, iArg, bNC, xFunc) \
  {nArg, SQLITE_FUNC_BUILTIN|SQLITE_FUNC_SLOCHNG|SQLITE_UTF8, \
   0, 0, xFunc, 0, 0, 0, #zName, {0} }
#define PURE_DATE(zName, nArg, iArg, bNC, xFunc) \
  {nArg, SQLITE_FUNC_BUILTIN|\
         SQLITE_FUNC_SLOCHNG|SQLITE_UTF8|SQLITE_FUNC_CONSTANT, \
   (void*)&sqlite3Config, 0, xFunc, 0, 0, 0, #zName, {0} }
#define FUNCTION2(zName, nArg, iArg, bNC, xFunc, extraFlags) \
  {nArg, SQLITE_FUNC_BUILTIN|\
   SQLITE_FUNC_CONSTANT|SQLITE_UTF8|(bNC*SQLITE_FUNC_NEEDCOLL)|extraFlags,\
   SQLITE_INT_TO_PTR(iArg), 0, xFunc, 0, 0, 0, #zName, {0} }
#define STR_FUNCTION(zName, nArg, pArg, bNC, xFunc) \
  {nArg, SQLITE_FUNC_BUILTIN|\
   SQLITE_FUNC_SLOCHNG|SQLITE_UTF8|(bNC*SQLITE_FUNC_NEEDCOLL), \
   pArg, 0, xFunc, 0, 0, 0, #zName, }
#define LIKEFUNC(zName, nArg, arg, flags) \
  {nArg, SQLITE_FUNC_BUILTIN|SQLITE_FUNC_CONSTANT|SQLITE_UTF8|flags, \
   (void *)arg, 0, likeFunc, 0, 0, 0, #zName, {0} }
#define WAGGREGATE(zName, nArg, arg, nc, xStep, xFinal, xValue, xInverse, f) \
  {nArg, SQLITE_FUNC_BUILTIN|SQLITE_UTF8|(nc*SQLITE_FUNC_NEEDCOLL)|f, \
   SQLITE_INT_TO_PTR(arg), 0, xStep,xFinal,xValue,xInverse,#zName, {0}}
#define INTERNAL_FUNCTION(zName, nArg, xFunc) \
  {nArg, SQLITE_FUNC_BUILTIN|\
   SQLITE_FUNC_INTERNAL|SQLITE_UTF8|SQLITE_FUNC_CONSTANT, \
   0, 0, xFunc, 0, 0, 0, #zName, {0} }


// TODO: move this to rust
#define sqlite3IsNumericAffinity(X)  ((X)>=SQLITE_AFF_NUMERIC)

#define IsView(X)           ((X)->eTabType==TABTYP_VIEW)
#define IsOrdinaryTable(X)  ((X)->eTabType==TABTYP_NORM)

/*
** Test to see whether or not a table is a virtual table.  This is
** done as a macro so that it will be optimized out when virtual
** table support is omitted from the build.
*/
#ifndef SQLITE_OMIT_VIRTUALTABLE
#  define IsVirtual(X)      ((X)->eTabType==TABTYP_VTAB)
#  define ExprIsVtab(X)  \
   ((X)->op==TK_COLUMN && (X)->y.pTab->eTabType==TABTYP_VTAB)
#else
#  define IsVirtual(X)      0
#  define ExprIsVtab(X)     0
#endif

/*
** Macros to determine if a column is hidden.  IsOrdinaryHiddenColumn()
** only works for non-virtual tables (ordinary tables and views) and is
** always false unless SQLITE_ENABLE_HIDDEN_COLUMNS is defined.  The
** IsHiddenColumn() macro is general purpose.
*/
#if defined(SQLITE_ENABLE_HIDDEN_COLUMNS)
#  define IsHiddenColumn(X)         (((X)->colFlags & COLFLAG_HIDDEN)!=0)
#  define IsOrdinaryHiddenColumn(X) (((X)->colFlags & COLFLAG_HIDDEN)!=0)
#elif !defined(SQLITE_OMIT_VIRTUALTABLE)
#  define IsHiddenColumn(X)         (((X)->colFlags & COLFLAG_HIDDEN)!=0)
#  define IsOrdinaryHiddenColumn(X) 0
#else
#  define IsHiddenColumn(X)         0
#  define IsOrdinaryHiddenColumn(X) 0
#endif


/* Does the table have a rowid */
#define HasRowid(X)     (((X)->tabFlags & TF_WithoutRowid)==0)
#define VisibleRowid(X) (((X)->tabFlags & TF_NoVisibleRowid)==0)

/*
** SQLite supports many different ways to resolve a constraint
** error.  ROLLBACK processing means that a constraint violation
** causes the operation in process to fail and for the current transaction
** to be rolled back.  ABORT processing means the operation in process
** fails and any prior changes from that one operation are backed out,
** but the transaction is not rolled back.  FAIL processing means that
** the operation in progress stops and returns an error code.  But prior
** changes due to the same operation are not backed out and no rollback
** occurs.  IGNORE means that the particular row that caused the constraint
** error is not inserted or updated.  Processing continues and no error
** is returned.  REPLACE means that preexisting database rows that caused
** a UNIQUE constraint violation are removed so that the new insert or
** update can proceed.  Processing continues and no error is reported.
** UPDATE applies to insert operations only and means that the insert
** is omitted and the DO UPDATE clause of an upsert is run instead.
**
** RESTRICT, SETNULL, SETDFLT, and CASCADE actions apply only to foreign keys.
** RESTRICT is the same as ABORT for IMMEDIATE foreign keys and the
** same as ROLLBACK for DEFERRED keys.  SETNULL means that the foreign
** key is set to NULL.  SETDFLT means that the foreign key is set
** to its default value.  CASCADE means that a DELETE or UPDATE of the
** referenced table row is propagated into the row that holds the
** foreign key.
**
** The OE_Default value is a place holder that means to use whatever
** conflict resolution algorthm is required from context.
**
** The following symbolic values are used to record which type
** of conflict resolution action to take.
*/
#define OE_None     0   /* There is no constraint to check */
#define OE_Rollback 1   /* Fail the operation and rollback the transaction */
#define OE_Abort    2   /* Back out changes but do no rollback transaction */
#define OE_Fail     3   /* Stop the operation but leave all prior changes */
#define OE_Ignore   4   /* Ignore the error. Do not do the INSERT or UPDATE */
#define OE_Replace  5   /* Delete existing record, then do INSERT or UPDATE */
#define OE_Update   6   /* Process as a DO UPDATE in an upsert */
#define OE_Restrict 7   /* OE_Abort for IMMEDIATE, OE_Rollback for DEFERRED */
#define OE_SetNull  8   /* Set the foreign key value to NULL */
#define OE_SetDflt  9   /* Set the foreign key value to its default */
#define OE_Cascade  10  /* Cascade the changes */
#define OE_Default  11  /* Do whatever the default action is */

/*
** Allowed bit values for entries in the KeyInfo.aSortFlags[] array.
*/
#define KEYINFO_ORDER_DESC    0x01    /* DESC sort order */
#define KEYINFO_ORDER_BIGNULL 0x02    /* NULL is larger than any other value */

/* Return true if index X is a PRIMARY KEY index */
#define IsPrimaryKeyIndex(X)  ((X)->idxType==SQLITE_IDXTYPE_PRIMARYKEY)

/* Return true if index X is a UNIQUE index */
#define IsUniqueIndex(X)      ((X)->onError!=OE_None)

/* The Index.aiColumn[] values are normally positive integer.  But
** there are some negative values that have special meaning:
*/
#define XN_ROWID     (-1)     /* Indexed column is the rowid */
#define XN_EXPR      (-2)     /* Indexed column is an expression */

/*
** Macros to compute aCol[] and aFunc[] register numbers.
**
** These macros should not be used prior to the call to 
** assignAggregateRegisters() that computes the value of pAggInfo->iFirstReg.
** The assert()s that are part of this macro verify that constraint.
*/
#define AggInfoColumnReg(A,I)  (assert((A)->iFirstReg),(A)->iFirstReg+(I))
#define AggInfoFuncReg(A,I)    \
                      (assert((A)->iFirstReg),(A)->iFirstReg+(A)->nColumn+(I))

/*
** The datatype ynVar is a signed integer, either 16-bit or 32-bit.
** Usually it is 16-bits.  But if SQLITE_MAX_VARIABLE_NUMBER is greater
** than 32767 we have to make it 32-bit.  16-bit is preferred because
** it uses less memory in the Expr object, which is a big memory user
** in systems with lots of prepared statements.  And few applications
** need more than about 10 or 20 variables.  But some extreme users want
** to have prepared statements with over 32766 variables, and for them
** the option is available (at compile-time).
*/
#if SQLITE_MAX_VARIABLE_NUMBER<32767
typedef i16 ynVar;
#else
typedef int ynVar;
#endif

/*
** Macros to determine the number of bytes required by a normal Expr
** struct, an Expr struct with the EP_Reduced flag set in Expr.flags
** and an Expr struct with the EP_TokenOnly flag set.
*/
#define EXPR_FULLSIZE           sizeof(Expr)           /* Full size */
#define EXPR_REDUCEDSIZE        offsetof(Expr,iTable)  /* Common features */
#define EXPR_TOKENONLYSIZE      offsetof(Expr,pLeft)   /* Fewer features */

/*
** Flags passed to the sqlite3ExprDup() function. See the header comment
** above sqlite3ExprDup() for details.
*/
#define EXPRDUP_REDUCE         0x0001  /* Used reduced-size Expr nodes */

/*
** True if the expression passed as an argument was a function with
** an OVER() clause (a window function).
*/
#ifdef SQLITE_OMIT_WINDOWFUNC
# define IsWindowFunc(p) 0
#else
# define IsWindowFunc(p) ( \
    ExprHasProperty((p), EP_WinFunc) && p->y.pWin->eFrmType!=TK_FILTER \
 )
#endif

/*
** Flags appropriate for the wctrlFlags parameter of sqlite3WhereBegin()
** and the WhereInfo.wctrlFlags member.
**
** Value constraints (enforced via assert()):
**     WHERE_USE_LIMIT  == SF_FixedLimit
*/
#define WHERE_ORDERBY_NORMAL   0x0000 /* No-op */
#define WHERE_ORDERBY_MIN      0x0001 /* ORDER BY processing for min() func */
#define WHERE_ORDERBY_MAX      0x0002 /* ORDER BY processing for max() func */
#define WHERE_ONEPASS_DESIRED  0x0004 /* Want to do one-pass UPDATE/DELETE */
#define WHERE_ONEPASS_MULTIROW 0x0008 /* ONEPASS is ok with multiple rows */
#define WHERE_DUPLICATES_OK    0x0010 /* Ok to return a row more than once */
#define WHERE_OR_SUBCLAUSE     0x0020 /* Processing a sub-WHERE as part of
                                      ** the OR optimization  */
#define WHERE_GROUPBY          0x0040 /* pOrderBy is really a GROUP BY */
#define WHERE_DISTINCTBY       0x0080 /* pOrderby is really a DISTINCT clause */
#define WHERE_WANT_DISTINCT    0x0100 /* All output needs to be distinct */
#define WHERE_SORTBYGROUP      0x0200 /* Support sqlite3WhereIsSorted() */
#define WHERE_AGG_DISTINCT     0x0400 /* Query is "SELECT agg(DISTINCT ...)" */
#define WHERE_ORDERBY_LIMIT    0x0800 /* ORDERBY+LIMIT on the inner loop */
#define WHERE_RIGHT_JOIN       0x1000 /* Processing a RIGHT JOIN */
                        /*     0x2000    not currently used */
#define WHERE_USE_LIMIT        0x4000 /* Use the LIMIT in cost estimates */
                        /*     0x8000    not currently used */


/* True if S exists and has SF_NestedFrom */
#define IsNestedFrom(S) ((S)!=0 && ((S)->selFlags&SF_NestedFrom)!=0)

/*
** The results of a SELECT can be distributed in several ways, as defined
** by one of the following macros.  The "SRT" prefix means "SELECT Result
** Type".
**
**     SRT_Union       Store results as a key in a temporary index
**                     identified by pDest->iSDParm.
**
**     SRT_Except      Remove results from the temporary index pDest->iSDParm.
**
**     SRT_Exists      Store a 1 in memory cell pDest->iSDParm if the result
**                     set is not empty.
**
**     SRT_Discard     Throw the results away.  This is used by SELECT
**                     statements within triggers whose only purpose is
**                     the side-effects of functions.
**
**     SRT_Output      Generate a row of output (using the OP_ResultRow
**                     opcode) for each row in the result set.
**
**     SRT_Mem         Only valid if the result is a single column.
**                     Store the first column of the first result row
**                     in register pDest->iSDParm then abandon the rest
**                     of the query.  This destination implies "LIMIT 1".
**
**     SRT_Set         The result must be a single column.  Store each
**                     row of result as the key in table pDest->iSDParm.
**                     Apply the affinity pDest->affSdst before storing
**                     results.  Used to implement "IN (SELECT ...)".
**
**     SRT_EphemTab    Create an temporary table pDest->iSDParm and store
**                     the result there. The cursor is left open after
**                     returning.  This is like SRT_Table except that
**                     this destination uses OP_OpenEphemeral to create
**                     the table first.
**
**     SRT_Coroutine   Generate a co-routine that returns a new row of
**                     results each time it is invoked.  The entry point
**                     of the co-routine is stored in register pDest->iSDParm
**                     and the result row is stored in pDest->nDest registers
**                     starting with pDest->iSdst.
**
**     SRT_Table       Store results in temporary table pDest->iSDParm.
**     SRT_Fifo        This is like SRT_EphemTab except that the table
**                     is assumed to already be open.  SRT_Fifo has
**                     the additional property of being able to ignore
**                     the ORDER BY clause.
**
**     SRT_DistFifo    Store results in a temporary table pDest->iSDParm.
**                     But also use temporary table pDest->iSDParm+1 as
**                     a record of all prior results and ignore any duplicate
**                     rows.  Name means:  "Distinct Fifo".
**
**     SRT_Queue       Store results in priority queue pDest->iSDParm (really
**                     an index).  Append a sequence number so that all entries
**                     are distinct.
**
**     SRT_DistQueue   Store results in priority queue pDest->iSDParm only if
**                     the same record has never been stored before.  The
**                     index at pDest->iSDParm+1 hold all prior stores.
**
**     SRT_Upfrom      Store results in the temporary table already opened by
**                     pDest->iSDParm. If (pDest->iSDParm<0), then the temp
**                     table is an intkey table - in this case the first
**                     column returned by the SELECT is used as the integer
**                     key. If (pDest->iSDParm>0), then the table is an index
**                     table. (pDest->iSDParm) is the number of key columns in
**                     each index record in this case.
*/
#define SRT_Union        1  /* Store result as keys in an index */
#define SRT_Except       2  /* Remove result from a UNION index */
#define SRT_Exists       3  /* Store 1 if the result is not empty */
#define SRT_Discard      4  /* Do not save the results anywhere */
#define SRT_DistFifo     5  /* Like SRT_Fifo, but unique results only */
#define SRT_DistQueue    6  /* Like SRT_Queue, but unique results only */

/* The DISTINCT clause is ignored for all of the above.  Not that
** IgnorableDistinct() implies IgnorableOrderby() */
#define IgnorableDistinct(X) ((X->eDest)<=SRT_DistQueue)

#define SRT_Queue        7  /* Store result in an queue */
#define SRT_Fifo         8  /* Store result as data with an automatic rowid */

/* The ORDER BY clause is ignored for all of the above */
#define IgnorableOrderby(X) ((X->eDest)<=SRT_Fifo)

#define SRT_Output       9  /* Output each row of result */
#define SRT_Mem         10  /* Store result in a memory cell */
#define SRT_Set         11  /* Store results as keys in an index */
#define SRT_EphemTab    12  /* Create transient tab and store like SRT_Table */
#define SRT_Coroutine   13  /* Generate a single row of result */
#define SRT_Table       14  /* Store result as data with an automatic rowid */
#define SRT_Upfrom      15  /* Store result as data with rowid */

/*
** The yDbMask datatype for the bitmask of all attached databases.
*/
#if SQLITE_MAX_ATTACHED>30
  typedef unsigned char yDbMask[(SQLITE_MAX_ATTACHED+9)/8];
# define DbMaskTest(M,I)    (((M)[(I)/8]&(1<<((I)&7)))!=0)
# define DbMaskZero(M)      memset((M),0,sizeof(M))
# define DbMaskSet(M,I)     (M)[(I)/8]|=(1<<((I)&7))
# define DbMaskAllZero(M)   sqlite3DbMaskAllZero(M)
# define DbMaskNonZero(M)   (sqlite3DbMaskAllZero(M)==0)
#else
  typedef unsigned int yDbMask;
# define DbMaskTest(M,I)    (((M)&(((yDbMask)1)<<(I)))!=0)
# define DbMaskZero(M)      ((M)=0)
# define DbMaskSet(M,I)     ((M)|=(((yDbMask)1)<<(I)))
# define DbMaskAllZero(M)   ((M)==0)
# define DbMaskNonZero(M)   ((M)!=0)
#endif


/*
** Sizes and pointers of various parts of the Parse object.
*/
#define PARSE_HDR(X)  (((char*)(X))+offsetof(Parse,zErrMsg))
#define PARSE_HDR_SZ (offsetof(Parse,aTempReg)-offsetof(Parse,zErrMsg)) /* Recursive part w/o aColCache*/
#define PARSE_RECURSE_SZ offsetof(Parse,sLastToken)    /* Recursive part */
#define PARSE_TAIL_SZ (sizeof(Parse)-PARSE_RECURSE_SZ) /* Non-recursive part */
#define PARSE_TAIL(X) (((char*)(X))+PARSE_RECURSE_SZ)  /* Pointer to tail */

/*
** Return true if currently inside an sqlite3_declare_vtab() call.
*/
#ifdef SQLITE_OMIT_VIRTUALTABLE
  #define IN_DECLARE_VTAB 0
#else
  #define IN_DECLARE_VTAB (pParse->eParseMode==PARSE_MODE_DECLARE_VTAB)
#endif

#if defined(SQLITE_OMIT_ALTERTABLE)
  #define IN_RENAME_OBJECT 0
#else
  #define IN_RENAME_OBJECT (pParse->eParseMode>=PARSE_MODE_RENAME)
#endif

#if defined(SQLITE_OMIT_VIRTUALTABLE) && defined(SQLITE_OMIT_ALTERTABLE)
  #define IN_SPECIAL_PARSE 0
#else
  #define IN_SPECIAL_PARSE (pParse->eParseMode!=PARSE_MODE_NORMAL)
#endif

/*
** Bitfield flags for P5 value in various opcodes.
**
** Value constraints (enforced via assert()):
**    OPFLAG_LENGTHARG    == SQLITE_FUNC_LENGTH
**    OPFLAG_TYPEOFARG    == SQLITE_FUNC_TYPEOF
**    OPFLAG_BULKCSR      == BTREE_BULKLOAD
**    OPFLAG_SEEKEQ       == BTREE_SEEK_EQ
**    OPFLAG_FORDELETE    == BTREE_FORDELETE
**    OPFLAG_SAVEPOSITION == BTREE_SAVEPOSITION
**    OPFLAG_AUXDELETE    == BTREE_AUXDELETE
*/
#define OPFLAG_NCHANGE       0x01    /* OP_Insert: Set to update db->nChange */
                                     /* Also used in P2 (not P5) of OP_Delete */
#define OPFLAG_NOCHNG        0x01    /* OP_VColumn nochange for UPDATE */
#define OPFLAG_EPHEM         0x01    /* OP_Column: Ephemeral output is ok */
#define OPFLAG_LASTROWID     0x20    /* Set to update db->lastRowid */
#define OPFLAG_ISUPDATE      0x04    /* This OP_Insert is an sql UPDATE */
#define OPFLAG_APPEND        0x08    /* This is likely to be an append */
#define OPFLAG_USESEEKRESULT 0x10    /* Try to avoid a seek in BtreeInsert() */
#define OPFLAG_ISNOOP        0x40    /* OP_Delete does pre-update-hook only */
#define OPFLAG_LENGTHARG     0x40    /* OP_Column only used for length() */
#define OPFLAG_TYPEOFARG     0x80    /* OP_Column only used for typeof() */
#define OPFLAG_BULKCSR       0x01    /* OP_Open** used to open bulk cursor */
#define OPFLAG_SEEKEQ        0x02    /* OP_Open** cursor uses EQ seek only */
#define OPFLAG_FORDELETE     0x08    /* OP_Open should use BTREE_FORDELETE */
#define OPFLAG_P2ISREG       0x10    /* P2 to OP_Open** is a register number */
#define OPFLAG_PERMUTE       0x01    /* OP_Compare: use the permutation */
#define OPFLAG_SAVEPOSITION  0x02    /* OP_Delete/Insert: save cursor pos */
#define OPFLAG_AUXDELETE     0x04    /* OP_Delete: index in a DELETE op */
#define OPFLAG_NOCHNG_MAGIC  0x6d    /* OP_MakeRecord: serialtype 10 is ok */
#define OPFLAG_PREFORMAT     0x80    /* OP_Insert uses preformatted cell */ 

/*
** An objected used to accumulate the text of a string where we
** do not necessarily know how big the string will be in the end.
*/
struct sqlite3_str {
  sqlite3 *db;         /* Optional database for lookaside.  Can be NULL */
  char *zText;         /* The string collected so far */
  u32  nAlloc;         /* Amount of space allocated in zText */
  u32  mxAlloc;        /* Maximum allowed allocation.  0 for no malloc usage */
  u32  nChar;          /* Length of the string so far */
  u8   accError;       /* SQLITE_NOMEM or SQLITE_TOOBIG */
  u8   printfFlags;    /* SQLITE_PRINTF flags below */
};
#define SQLITE_PRINTF_INTERNAL 0x01  /* Internal-use-only converters allowed */
#define SQLITE_PRINTF_SQLFUNC  0x02  /* SQL function arguments to VXPrintf */
#define SQLITE_PRINTF_MALLOCED 0x04  /* True if xText is allocated space */

#define isMalloced(X)  (((X)->printfFlags & SQLITE_PRINTF_MALLOCED)!=0)


/*
** A pointer to this structure is used to communicate information
** from sqlite3Init and OP_ParseSchema into the sqlite3InitCallback.
*/
typedef struct {
  sqlite3 *db;        /* The database being initialized */
  char **pzErrMsg;    /* Error message stored here */
  int iDb;            /* 0 for main database.  1 for TEMP, 2.. for ATTACHed */
  int rc;             /* Result code stored here */
  u32 mInitFlags;     /* Flags controlling error messages */
  u32 nInitRow;       /* Number of rows processed */
  Pgno mxPage;        /* Maximum page number.  0 for no limit. */
} InitData;

/*
** Allowed values for mInitFlags
*/
#define INITFLAG_AlterMask     0x0003  /* Types of ALTER */
#define INITFLAG_AlterRename   0x0001  /* Reparse after a RENAME */
#define INITFLAG_AlterDrop     0x0002  /* Reparse after a DROP COLUMN */
#define INITFLAG_AlterAdd      0x0003  /* Reparse after an ADD COLUMN */

/* Tuning parameters are set using SQLITE_TESTCTRL_TUNE and are controlled
** on debug-builds of the CLI using ".testctrl tune ID VALUE".  Tuning
** parameters are for temporary use during development, to help find
** optimial values for parameters in the query planner.  The should not
** be used on trunk check-ins.  They are a temporary mechanism available
** for transient development builds only.
**
** Tuning parameters are numbered starting with 1.
*/
#define SQLITE_NTUNE  6             /* Should be zero for all trunk check-ins */
#ifdef SQLITE_DEBUG
# define Tuning(X)  (sqlite3Config.aTune[(X)-1])
#else
# define Tuning(X)  0
#endif

/*
** Structure containing global configuration data for the SQLite library.
**
** This structure also contains some state information.
*/
struct Sqlite3Config {
  int bMemstat;                     /* True to enable memory status */
  u8 bCoreMutex;                    /* True to enable core mutexing */
  u8 bFullMutex;                    /* True to enable full mutexing */
  u8 bOpenUri;                      /* True to interpret filenames as URIs */
  u8 bUseCis;                       /* Use covering indices for full-scans */
  u8 bSmallMalloc;                  /* Avoid large memory allocations if true */
  u8 bExtraSchemaChecks;            /* Verify type,name,tbl_name in schema */
  int mxStrlen;                     /* Maximum string length */
  int neverCorrupt;                 /* Database is always well-formed */
  int szLookaside;                  /* Default lookaside buffer size */
  int nLookaside;                   /* Default lookaside buffer count */
  int nStmtSpill;                   /* Stmt-journal spill-to-disk threshold */
  sqlite3_mem_methods m;            /* Low-level memory allocation interface */
  sqlite3_mutex_methods mutex;      /* Low-level mutex interface */
  sqlite3_pcache_methods2 pcache2;  /* Low-level page-cache interface */
  void *pHeap;                      /* Heap storage space */
  int nHeap;                        /* Size of pHeap[] */
  int mnReq, mxReq;                 /* Min and max heap requests sizes */
  sqlite3_int64 szMmap;             /* mmap() space per open file */
  sqlite3_int64 mxMmap;             /* Maximum value for szMmap */
  void *pPage;                      /* Page cache memory */
  int szPage;                       /* Size of each page in pPage[] */
  int nPage;                        /* Number of pages in pPage[] */
  int mxParserStack;                /* maximum depth of the parser stack */
  int sharedCacheEnabled;           /* true if shared-cache mode enabled */
  u32 szPma;                        /* Maximum Sorter PMA size */
  /* The above might be initialized to non-zero.  The following need to always
  ** initially be zero, however. */
  int isInit;                       /* True after initialization has finished */
  int inProgress;                   /* True while initialization in progress */
  int isMutexInit;                  /* True after mutexes are initialized */
  int isMallocInit;                 /* True after malloc is initialized */
  int isPCacheInit;                 /* True after malloc is initialized */
  int nRefInitMutex;                /* Number of users of pInitMutex */
  sqlite3_mutex *pInitMutex;        /* Mutex used by sqlite3_initialize() */
  void (*xLog)(void*,int,const char*); /* Function for logging */
  void *pLogArg;                       /* First argument to xLog() */
#ifdef SQLITE_ENABLE_SQLLOG
  void(*xSqllog)(void*,sqlite3*,const char*, int);
  void *pSqllogArg;
#endif
#ifdef SQLITE_VDBE_COVERAGE
  /* The following callback (if not NULL) is invoked on every VDBE branch
  ** operation.  Set the callback using SQLITE_TESTCTRL_VDBE_COVERAGE.
  */
  void (*xVdbeBranch)(void*,unsigned iSrcLine,u8 eThis,u8 eMx);  /* Callback */
  void *pVdbeBranchArg;                                     /* 1st argument */
#endif
#ifndef SQLITE_OMIT_DESERIALIZE
  sqlite3_int64 mxMemdbSize;        /* Default max memdb size */
#endif
#ifndef SQLITE_UNTESTABLE
  int (*xTestCallback)(int);        /* Invoked by sqlite3FaultSim() */
#endif
  int bLocaltimeFault;              /* True to fail localtime() calls */
  int (*xAltLocaltime)(const void*,void*); /* Alternative localtime() routine */
  int iOnceResetThreshold;          /* When to reset OP_Once counters */
  u32 szSorterRef;                  /* Min size in bytes to use sorter-refs */
  unsigned int iPrngSeed;           /* Alternative fixed seed for the PRNG */
  /* vvvv--- must be last ---vvv */
#ifdef SQLITE_DEBUG
  sqlite3_int64 aTune[SQLITE_NTUNE]; /* Tuning parameters */
#endif
};

/*
** This macro is used inside of assert() statements to indicate that
** the assert is only valid on a well-formed database.  Instead of:
**
**     assert( X );
**
** One writes:
**
**     assert( X || CORRUPT_DB );
**
** CORRUPT_DB is true during normal operation.  CORRUPT_DB does not indicate
** that the database is definitely corrupt, only that it might be corrupt.
** For most test cases, CORRUPT_DB is set to false using a special
** sqlite3_test_control().  This enables assert() statements to prove
** things that are always true for well-formed databases.
*/
#define CORRUPT_DB  (sqlite3Config.neverCorrupt==0)

/*
** Context pointer passed down through the tree-walk.
*/
struct Walker {
  Parse *pParse;                            /* Parser context.  */
  int (*xExprCallback)(Walker*, Expr*);     /* Callback for expressions */
  int (*xSelectCallback)(Walker*,Select*);  /* Callback for SELECTs */
  void (*xSelectCallback2)(Walker*,Select*);/* Second callback for SELECTs */
  int walkerDepth;                          /* Number of subqueries */
  u16 eCode;                                /* A small processing code */
  union {                                   /* Extra data for callback */
    NameContext *pNC;                         /* Naming context */
    int n;                                    /* A counter */
    int iCur;                                 /* A cursor number */
    SrcList *pSrcList;                        /* FROM clause */
    struct CCurHint *pCCurHint;               /* Used by codeCursorHint() */
    struct RefSrcList *pRefSrcList;           /* sqlite3ReferencesSrcList() */
    int *aiCol;                               /* array of column indexes */
    struct IdxCover *pIdxCover;               /* Check for index coverage */
    ExprList *pGroupBy;                       /* GROUP BY clause */
    Select *pSelect;                          /* HAVING to WHERE clause ctx */
    struct WindowRewrite *pRewrite;           /* Window rewrite context */
    struct WhereConst *pConst;                /* WHERE clause constants */
    struct RenameCtx *pRename;                /* RENAME COLUMN context */
    struct Table *pTab;                       /* Table of generated column */
    struct CoveringIndexCheck *pCovIdxCk;     /* Check for covering index */
    SrcItem *pSrcItem;                        /* A single FROM clause item */
    DbFixer *pFix;                            /* See sqlite3FixSelect() */
  } u;
};

/*
** The following structure contains information used by the sqliteFix...
** routines as they walk the parse tree to make database references
** explicit.
*/
struct DbFixer {
  Parse *pParse;      /* The parsing context.  Error messages written here */
  Walker w;           /* Walker object */
  Schema *pSchema;    /* Fix items to this schema */
  u8 bTemp;           /* True for TEMP schema entries */
  const char *zDb;    /* Make sure all objects are contained in this database */
  const char *zType;  /* Type of the container - used for error messages */
  const Token *pName; /* Name of the container - used for error messages */
};

/* Forward declarations */
int sqlite3WalkExpr(Walker*, Expr*);
int sqlite3WalkExprList(Walker*, ExprList*);
int sqlite3WalkSelect(Walker*, Select*);
int sqlite3WalkSelectExpr(Walker*, Select*);
int sqlite3WalkSelectFrom(Walker*, Select*);
int sqlite3ExprWalkNoop(Walker*, Expr*);
int sqlite3SelectWalkNoop(Walker*, Select*);
int sqlite3SelectWalkFail(Walker*, Select*);
int sqlite3WalkerDepthIncrease(Walker*,Select*);
void sqlite3WalkerDepthDecrease(Walker*,Select*);
void sqlite3WalkWinDefnDummyCallback(Walker*,Select*);

#ifdef SQLITE_DEBUG
void sqlite3SelectWalkAssert2(Walker*, Select*);
#endif

#ifndef SQLITE_OMIT_CTE
void sqlite3SelectPopWith(Walker*, Select*);
#else
# define sqlite3SelectPopWith 0
#endif

/*
** Return code from the parse-tree walking primitives and their
** callbacks.
*/
#define WRC_Continue    0   /* Continue down into children */
#define WRC_Prune       1   /* Omit children but continue walking siblings */
#define WRC_Abort       2   /* Abandon the tree walk */

#ifdef SQLITE_DEBUG
/*
** An instance of the TreeView object is used for printing the content of
** data structures on sqlite3DebugPrintf() using a tree-like view.
*/
struct TreeView {
  int iLevel;             /* Which level of the tree we are on */
  u8  bLine[100];         /* Draw vertical in column i if bLine[i] is true */
};
#endif /* SQLITE_DEBUG */

#ifndef SQLITE_OMIT_WINDOWFUNC
void sqlite3WindowDelete(sqlite3*, Window*);
void sqlite3WindowUnlinkFromSelect(Window*);
void sqlite3WindowListDelete(sqlite3 *db, Window *p);
Window *sqlite3WindowAlloc(Parse*, int, int, Expr*, int , Expr*, u8);
void sqlite3WindowAttach(Parse*, Expr*, Window*);
void sqlite3WindowLink(Select *pSel, Window *pWin);
int sqlite3WindowCompare(const Parse*, const Window*, const Window*, int);
void sqlite3WindowCodeInit(Parse*, Select*);
void sqlite3WindowCodeStep(Parse*, Select*, WhereInfo*, int, int);
int sqlite3WindowRewrite(Parse*, Select*);
void sqlite3WindowUpdate(Parse*, Window*, Window*, FuncDef*);
Window *sqlite3WindowDup(sqlite3 *db, Expr *pOwner, Window *p);
Window *sqlite3WindowListDup(sqlite3 *db, Window *p);
void sqlite3WindowFunctions(void);
void sqlite3WindowChain(Parse*, Window*, Window*);
Window *sqlite3WindowAssemble(Parse*, Window*, ExprList*, ExprList*, Token*);
#else
# define sqlite3WindowDelete(a,b)
# define sqlite3WindowFunctions()
# define sqlite3WindowAttach(a,b,c)
#endif

/*
** Assuming zIn points to the first byte of a UTF-8 character,
** advance zIn to point to the first byte of the next UTF-8 character.
*/
#define SQLITE_SKIP_UTF8(zIn) {                        \
  if( (*(zIn++))>=0xc0 ){                              \
    while( (*zIn & 0xc0)==0x80 ){ zIn++; }             \
  }                                                    \
}

/*
** The SQLITE_*_BKPT macros are substitutes for the error codes with
** the same name but without the _BKPT suffix.  These macros invoke
** routines that report the line-number on which the error originated
** using sqlite3_log().  The routines also provide a convenient place
** to set a debugger breakpoint.
*/
int sqlite3ReportError(int iErr, int lineno, const char *zType);
int sqlite3CorruptError(int);
int sqlite3MisuseError(int);
int sqlite3CantopenError(int);
#define SQLITE_CORRUPT_BKPT sqlite3CorruptError(__LINE__)
#define SQLITE_MISUSE_BKPT sqlite3MisuseError(__LINE__)
#define SQLITE_CANTOPEN_BKPT sqlite3CantopenError(__LINE__)
#ifdef SQLITE_DEBUG
  int sqlite3NomemError(int);
  int sqlite3IoerrnomemError(int);
# define SQLITE_NOMEM_BKPT sqlite3NomemError(__LINE__)
# define SQLITE_IOERR_NOMEM_BKPT sqlite3IoerrnomemError(__LINE__)
#else
# define SQLITE_NOMEM_BKPT SQLITE_NOMEM
# define SQLITE_IOERR_NOMEM_BKPT SQLITE_IOERR_NOMEM
#endif
#if defined(SQLITE_DEBUG) || defined(SQLITE_ENABLE_CORRUPT_PGNO)
  int sqlite3CorruptPgnoError(int,Pgno);
# define SQLITE_CORRUPT_PGNO(P) sqlite3CorruptPgnoError(__LINE__,(P))
#else
# define SQLITE_CORRUPT_PGNO(P) sqlite3CorruptError(__LINE__)
#endif

/*
** FTS3 and FTS4 both require virtual table support
*/
#if defined(SQLITE_OMIT_VIRTUALTABLE)
# undef SQLITE_ENABLE_FTS3
# undef SQLITE_ENABLE_FTS4
#endif

/*
** FTS4 is really an extension for FTS3.  It is enabled using the
** SQLITE_ENABLE_FTS3 macro.  But to avoid confusion we also call
** the SQLITE_ENABLE_FTS4 macro to serve as an alias for SQLITE_ENABLE_FTS3.
*/
#if defined(SQLITE_ENABLE_FTS4) && !defined(SQLITE_ENABLE_FTS3)
# define SQLITE_ENABLE_FTS3 1
#endif

/*
** The ctype.h header is needed for non-ASCII systems.  It is also
** needed by FTS3 when FTS3 is included in the amalgamation.
*/
#if defined(SQLITE_ENABLE_FTS3) && defined(SQLITE_AMALGAMATION)
# include <ctype.h>
#endif

int sqlite3IsIdChar(u8);

/*
** Internal function prototypes
*/
int sqlite3Strlen30(const char*);
#define sqlite3Strlen30NN(C) (strlen(C)&0x3fffffff)
char *sqlite3ColumnType(Column*,char*);
#define sqlite3StrNICmp sqlite3_strnicmp

int sqlite3MallocInit(void);
void sqlite3MallocEnd(void);
void *sqlite3MallocZero(u64);
void *sqlite3Realloc(void*, u64);
void *sqlite3PageMalloc(int);
void sqlite3PageFree(void*);
void sqlite3MemSetDefault(void);
#ifndef SQLITE_UNTESTABLE
void sqlite3BenignMallocHooks(void (*)(void), void (*)(void));
#endif
int sqlite3HeapNearlyFull(void);

/*
** On systems with ample stack space and that support alloca(), make
** use of alloca() to obtain space for large automatic objects.  By default,
** obtain space from malloc().
**
** The alloca() routine never returns NULL.  This will cause code paths
** that deal with sqlite3StackAlloc() failures to be unreachable.
*/
#ifdef SQLITE_USE_ALLOCA
# define sqlite3StackAllocRaw(D,N)   alloca(N)
# define sqlite3StackAllocRawNN(D,N) alloca(N)
# define sqlite3StackFree(D,P)
# define sqlite3StackFreeNN(D,P)
#else
# define sqlite3StackAllocRaw(D,N)   sqlite3DbMallocRaw(D,N)
# define sqlite3StackAllocRawNN(D,N) sqlite3DbMallocRawNN(D,N)
# define sqlite3StackFree(D,P)       sqlite3DbFree(D,P)
# define sqlite3StackFreeNN(D,P)     sqlite3DbFreeNN(D,P)
#endif

/* Do not allow both MEMSYS5 and MEMSYS3 to be defined together.  If they
** are, disable MEMSYS3
*/
#ifdef SQLITE_ENABLE_MEMSYS5
const sqlite3_mem_methods *sqlite3MemGetMemsys5(void);
#undef SQLITE_ENABLE_MEMSYS3
#endif
#ifdef SQLITE_ENABLE_MEMSYS3
const sqlite3_mem_methods *sqlite3MemGetMemsys3(void);
#endif


#ifndef SQLITE_MUTEX_OMIT
  sqlite3_mutex_methods const *sqlite3DefaultMutex(void);
  sqlite3_mutex_methods const *sqlite3NoopMutex(void);
  sqlite3_mutex *sqlite3MutexAlloc(int);
  int sqlite3MutexInit(void);
  int sqlite3MutexEnd(void);
#endif
#if !defined(SQLITE_MUTEX_OMIT) && !defined(SQLITE_MUTEX_NOOP)
  void sqlite3MemoryBarrier(void);
#else
# define sqlite3MemoryBarrier()
#endif

sqlite3_int64 sqlite3StatusValue(int);
void sqlite3StatusUp(int, int);
void sqlite3StatusDown(int, int);
void sqlite3StatusHighwater(int, int);
int sqlite3LookasideUsed(sqlite3*,int*);

/* Access to mutexes used by sqlite3_status() */
sqlite3_mutex *sqlite3Pcache1Mutex(void);
sqlite3_mutex *sqlite3MallocMutex(void);

#if defined(SQLITE_ENABLE_MULTITHREADED_CHECKS) && !defined(SQLITE_MUTEX_OMIT)
void sqlite3MutexWarnOnContention(sqlite3_mutex*);
#else
# define sqlite3MutexWarnOnContention(x)
#endif

#ifndef SQLITE_OMIT_FLOATING_POINT
# define EXP754 (((u64)0x7ff)<<52)
# define MAN754 ((((u64)1)<<52)-1)
# define IsNaN(X) (((X)&EXP754)==EXP754 && ((X)&MAN754)!=0)
  int sqlite3IsNaN(double);
#else
# define IsNaN(X)         0
# define sqlite3IsNaN(X)  0
#endif

/*
** An instance of the following structure holds information about SQL
** functions arguments that are the parameters to the printf() function.
*/
struct PrintfArguments {
  int nArg;                /* Total number of arguments */
  int nUsed;               /* Number of arguments used so far */
  sqlite3_value **apArg;   /* The argument values */
};

char *sqlite3MPrintf(sqlite3*,const char*, ...);
char *sqlite3VMPrintf(sqlite3*,const char*, va_list);
#if defined(SQLITE_DEBUG) || defined(SQLITE_HAVE_OS_TRACE)
  void sqlite3DebugPrintf(const char*, ...);
#endif
#if defined(SQLITE_TEST)
  void *sqlite3TestTextToPtr(const char*);
#endif

#if defined(SQLITE_DEBUG)
  void sqlite3TreeViewLine(TreeView*, const char *zFormat, ...);
  void sqlite3TreeViewExpr(TreeView*, const Expr*, u8);
  void sqlite3TreeViewBareExprList(TreeView*, const ExprList*, const char*);
  void sqlite3TreeViewExprList(TreeView*, const ExprList*, u8, const char*);
  void sqlite3TreeViewBareIdList(TreeView*, const IdList*, const char*);
  void sqlite3TreeViewIdList(TreeView*, const IdList*, u8, const char*);
  void sqlite3TreeViewColumnList(TreeView*, const Column*, int, u8);
  void sqlite3TreeViewSrcList(TreeView*, const SrcList*);
  void sqlite3TreeViewSelect(TreeView*, const Select*, u8);
  void sqlite3TreeViewWith(TreeView*, const With*, u8);
  void sqlite3TreeViewUpsert(TreeView*, const Upsert*, u8);
#if TREETRACE_ENABLED
  void sqlite3TreeViewDelete(const With*, const SrcList*, const Expr*,
                             const ExprList*,const Expr*, const Trigger*);
  void sqlite3TreeViewInsert(const With*, const SrcList*,
                             const IdList*, const Select*, const ExprList*,
                             int, const Upsert*, const Trigger*);
  void sqlite3TreeViewUpdate(const With*, const SrcList*, const ExprList*,
                             const Expr*, int, const ExprList*, const Expr*,
                             const Upsert*, const Trigger*);
#endif
#ifndef SQLITE_OMIT_TRIGGER
  void sqlite3TreeViewTriggerStep(TreeView*, const TriggerStep*, u8, u8);
  void sqlite3TreeViewTrigger(TreeView*, const Trigger*, u8, u8);
#endif
#ifndef SQLITE_OMIT_WINDOWFUNC
  void sqlite3TreeViewWindow(TreeView*, const Window*, u8);
  void sqlite3TreeViewWinFunc(TreeView*, const Window*, u8);
#endif
  void sqlite3ShowExpr(const Expr*);
  void sqlite3ShowExprList(const ExprList*);
  void sqlite3ShowIdList(const IdList*);
  void sqlite3ShowSrcList(const SrcList*);
  void sqlite3ShowSelect(const Select*);
  void sqlite3ShowWith(const With*);
  void sqlite3ShowUpsert(const Upsert*);
#ifndef SQLITE_OMIT_TRIGGER
  void sqlite3ShowTriggerStep(const TriggerStep*);
  void sqlite3ShowTriggerStepList(const TriggerStep*);
  void sqlite3ShowTrigger(const Trigger*);
  void sqlite3ShowTriggerList(const Trigger*);
#endif
#ifndef SQLITE_OMIT_WINDOWFUNC
  void sqlite3ShowWindow(const Window*);
  void sqlite3ShowWinFunc(const Window*);
#endif
#endif

void sqlite3SetString(char **, sqlite3*, const char*);
void sqlite3ProgressCheck(Parse*);
void sqlite3ErrorMsg(Parse*, const char*, ...);
int sqlite3ErrorToParser(sqlite3*,int);
void sqlite3DequoteToken(Token*);
void sqlite3TokenInit(Token*,char*);
int sqlite3KeywordCode(const unsigned char*, int);
int sqlite3RunParser(Parse*, const char*);
void sqlite3FinishCoding(Parse*);
void sqlite3ReleaseTempReg(Parse*,int);
void sqlite3ReleaseTempRange(Parse*,int,int);
#ifdef SQLITE_DEBUG
int sqlite3NoTempsInRange(Parse*,int,int);
#endif
Expr *sqlite3ExprAlloc(sqlite3*,int,const Token*,int);
Expr *sqlite3Expr(sqlite3*,int,const char*);
void sqlite3ExprAttachSubtrees(sqlite3*,Expr*,Expr*,Expr*);
Expr *sqlite3PExpr(Parse*, int, Expr*, Expr*);
void sqlite3PExprAddSelect(Parse*, Expr*, Select*);
Expr *sqlite3ExprAnd(Parse*,Expr*, Expr*);
Expr *sqlite3ExprSimplifiedAndOr(Expr*);
Expr *sqlite3ExprFunction(Parse*,ExprList*, const Token*, int);
void sqlite3ExprFunctionUsable(Parse*,const Expr*,const FuncDef*);
void sqlite3ExprAssignVarNumber(Parse*, Expr*, u32);
void sqlite3ExprDelete(sqlite3*, Expr*);
void sqlite3ExprDeferredDelete(Parse*, Expr*);
void sqlite3ExprUnmapAndDelete(Parse*, Expr*);
ExprList *sqlite3ExprListAppend(Parse*,ExprList*,Expr*);
ExprList *sqlite3ExprListAppendVector(Parse*,ExprList*,IdList*,Expr*);
Select *sqlite3ExprListToValues(Parse*, int, ExprList*);
void sqlite3ExprListSetSortOrder(ExprList*,int,int);
void sqlite3ExprListSetName(Parse*,ExprList*,const Token*,int);
void sqlite3ExprListSetSpan(Parse*,ExprList*,const char*,const char*);
void sqlite3ExprListDelete(sqlite3*, ExprList*);
u32 sqlite3ExprListFlags(const ExprList*);
int sqlite3IndexHasDuplicateRootPage(Index*);
int sqlite3Init(sqlite3*, char**);
int sqlite3InitCallback(void*, int, char**, char**);
int sqlite3InitOne(sqlite3*, int, char**, u32);
void sqlite3Pragma(Parse*,Token*,Token*,Token*,int);
#ifndef SQLITE_OMIT_VIRTUALTABLE
Module *sqlite3PragmaVtabRegister(sqlite3*,const char *zName);
#endif
void sqlite3ResetAllSchemasOfConnection(sqlite3*);
void sqlite3ResetOneSchema(sqlite3*,int);
void sqlite3CollapseDatabaseArray(sqlite3*);
void sqlite3CommitInternalChanges(sqlite3*);
void sqlite3ColumnSetExpr(Parse*,Table*,Column*,Expr*);
Expr *sqlite3ColumnExpr(Table*,Column*);
void sqlite3ColumnSetColl(sqlite3*,Column*,const char*zColl);
void sqlite3DeleteColumnNames(sqlite3*,Table*);
void sqlite3GenerateColumnNames(Parse *pParse, Select *pSelect);
int sqlite3ColumnsFromExprList(Parse*,ExprList*,i16*,Column**);
void sqlite3SubqueryColumnTypes(Parse*,Table*,Select*,char);
Table *sqlite3ResultSetOfSelect(Parse*,Select*,char);
void sqlite3OpenSchemaTable(Parse *, int);
Index *sqlite3PrimaryKeyIndex(Table*);
i16 sqlite3TableColumnToIndex(Index*, i16);
#ifdef SQLITE_OMIT_GENERATED_COLUMNS
# define sqlite3TableColumnToStorage(T,X) (X)  /* No-op pass-through */
# define sqlite3StorageColumnToTable(T,X) (X)  /* No-op pass-through */
#else
  i16 sqlite3TableColumnToStorage(Table*, i16);
  i16 sqlite3StorageColumnToTable(Table*, i16);
#endif
void sqlite3StartTable(Parse*,Token*,Token*,int,int,int,int);
#if SQLITE_ENABLE_HIDDEN_COLUMNS
  void sqlite3ColumnPropertiesFromName(Table*, Column*);
#else
# define sqlite3ColumnPropertiesFromName(T,C) /* no-op */
#endif
void sqlite3AddColumn(Parse*,Token,Token);
void sqlite3AddNotNull(Parse*, int);
void sqlite3AddPrimaryKey(Parse*, ExprList*, int, int, int);
void sqlite3AddCheckConstraint(Parse*, Expr*, const char*, const char*);
void sqlite3AddDefaultValue(Parse*,Expr*,const char*,const char*);
void sqlite3AddCollateType(Parse*, Token*);
void sqlite3AddGenerated(Parse*,Expr*,Token*);
void sqlite3EndTable(Parse*,Token*,Token*,u32,Select*);
void sqlite3AddReturning(Parse*,ExprList*);
int sqlite3ParseUri(const char*,const char*,unsigned int*,
                    sqlite3_vfs**,char**,char **);
#define sqlite3CodecQueryParameters(A,B,C) 0
Btree *sqlite3DbNameToBtree(sqlite3*,const char*);

#ifdef SQLITE_UNTESTABLE
# define sqlite3FaultSim(X) SQLITE_OK
#else
  int sqlite3FaultSim(int);
#endif

Bitvec *sqlite3BitvecCreate(u32);
int sqlite3BitvecSet(Bitvec*, u32);
void sqlite3BitvecDestroy(Bitvec*);

RowSet *sqlite3RowSetInit(sqlite3*);
void sqlite3RowSetDelete(void*);
void sqlite3RowSetClear(void*);
void sqlite3RowSetInsert(RowSet*, i64);
int sqlite3RowSetTest(RowSet*, int iBatch, i64);
int sqlite3RowSetNext(RowSet*, i64*);

void sqlite3CreateView(Parse*,Token*,Token*,Token*,ExprList*,Select*,int,int);

#if !defined(SQLITE_OMIT_VIEW) || !defined(SQLITE_OMIT_VIRTUALTABLE)
  int sqlite3ViewGetColumnNames(Parse*,Table*);
#else
# define sqlite3ViewGetColumnNames(A,B) 0
#endif

#if SQLITE_MAX_ATTACHED>30
  int sqlite3DbMaskAllZero(yDbMask);
#endif
void sqlite3DropTable(Parse*, SrcList*, int, int);
void sqlite3CodeDropTable(Parse*, Table*, int, int);
void sqlite3DeleteTable(sqlite3*, Table*);
void sqlite3FreeIndex(sqlite3*, Index*);
#ifndef SQLITE_OMIT_AUTOINCREMENT
  void sqlite3AutoincrementBegin(Parse *pParse);
  void sqlite3AutoincrementEnd(Parse *pParse);
#else
# define sqlite3AutoincrementBegin(X)
# define sqlite3AutoincrementEnd(X)
#endif
void sqlite3Insert(Parse*, SrcList*, Select*, IdList*, int, Upsert*);
#ifndef SQLITE_OMIT_GENERATED_COLUMNS
  void sqlite3ComputeGeneratedColumns(Parse*, int, Table*);
#endif
void *sqlite3ArrayAllocate(sqlite3*,void*,int,int*,int*);
SrcList *sqlite3SrcListEnlarge(Parse*, SrcList*, int, int);
SrcList *sqlite3SrcListAppendList(Parse *pParse, SrcList *p1, SrcList *p2);
SrcList *sqlite3SrcListAppend(Parse*, SrcList*, Token*, Token*);
SrcList *sqlite3SrcListAppendFromTerm(Parse*, SrcList*, Token*, Token*,
                                      Token*, Select*, OnOrUsing*);
void sqlite3SrcListIndexedBy(Parse *, SrcList *, Token *);
void sqlite3SrcListFuncArgs(Parse*, SrcList*, ExprList*);
int sqlite3IndexedByLookup(Parse *, SrcItem *);
void sqlite3SrcListShiftJoinType(Parse*,SrcList*);
void sqlite3SrcListAssignCursors(Parse*, SrcList*);
void sqlite3ClearOnOrUsing(sqlite3*, OnOrUsing*);
void sqlite3SrcListDelete(sqlite3*, SrcList*);
Index *sqlite3AllocateIndexObject(sqlite3*,i16,int,char**);
void sqlite3CreateIndex(Parse*,Token*,Token*,SrcList*,ExprList*,int,Token*,
                          Expr*, int, int, u8);
void sqlite3DropIndex(Parse*, SrcList*, int);
int sqlite3Select(Parse*, Select*, SelectDest*);
Select *sqlite3SelectNew(Parse*,ExprList*,SrcList*,Expr*,ExprList*,
                         Expr*,ExprList*,u32,Expr*);
void sqlite3SelectDelete(sqlite3*, Select*);
Table *sqlite3SrcListLookup(Parse*, SrcList*);
int sqlite3IsReadOnly(Parse*, Table*, int);
void sqlite3OpenTable(Parse*, int iCur, int iDb, Table*, int);
#if defined(SQLITE_ENABLE_UPDATE_DELETE_LIMIT) && !defined(SQLITE_OMIT_SUBQUERY)
Expr *sqlite3LimitWhere(Parse*,SrcList*,Expr*,ExprList*,Expr*,char*);
#endif
void sqlite3CodeChangeCount(Vdbe*,int,const char*);
void sqlite3DeleteFrom(Parse*, SrcList*, Expr*, ExprList*, Expr*);
void sqlite3Update(Parse*, SrcList*, ExprList*,Expr*,int,ExprList*,Expr*,
                   Upsert*);
WhereInfo *sqlite3WhereBegin(Parse*,SrcList*,Expr*,ExprList*,
                             ExprList*,Select*,u16,int);
void sqlite3WhereEnd(WhereInfo*);
LogEst sqlite3WhereOutputRowCount(WhereInfo*);
int sqlite3WhereIsDistinct(WhereInfo*);
int sqlite3WhereIsOrdered(WhereInfo*);
int sqlite3WhereOrderByLimitOptLabel(WhereInfo*);
void sqlite3WhereMinMaxOptEarlyOut(Vdbe*,WhereInfo*);
int sqlite3WhereIsSorted(WhereInfo*);
int sqlite3WhereContinueLabel(WhereInfo*);
int sqlite3WhereBreakLabel(WhereInfo*);
int sqlite3WhereOkOnePass(WhereInfo*, int*);
#define ONEPASS_OFF      0        /* Use of ONEPASS not allowed */
#define ONEPASS_SINGLE   1        /* ONEPASS valid for a single row update */
#define ONEPASS_MULTI    2        /* ONEPASS is valid for multiple rows */
int sqlite3WhereUsesDeferredSeek(WhereInfo*);
void sqlite3ExprCodeLoadIndexColumn(Parse*, Index*, int, int, int);
int sqlite3ExprCodeGetColumn(Parse*, Table*, int, int, int, u8);
void sqlite3ExprCodeGetColumnOfTable(Vdbe*, Table*, int, int, int);
void sqlite3ExprCodeMove(Parse*, int, int, int);
void sqlite3ExprCode(Parse*, Expr*, int);
#ifndef SQLITE_OMIT_GENERATED_COLUMNS
void sqlite3ExprCodeGeneratedColumn(Parse*, Table*, Column*, int);
#endif
void sqlite3ExprCodeCopy(Parse*, Expr*, int);
void sqlite3ExprCodeFactorable(Parse*, Expr*, int);
int sqlite3ExprCodeRunJustOnce(Parse*, Expr*, int);
int sqlite3ExprCodeTemp(Parse*, Expr*, int*);
int sqlite3ExprCodeTarget(Parse*, Expr*, int);
int sqlite3ExprCodeExprList(Parse*, ExprList*, int, int, u8);
#define SQLITE_ECEL_DUP      0x01  /* Deep, not shallow copies */
#define SQLITE_ECEL_FACTOR   0x02  /* Factor out constant terms */
#define SQLITE_ECEL_REF      0x04  /* Use ExprList.u.x.iOrderByCol */
#define SQLITE_ECEL_OMITREF  0x08  /* Omit if ExprList.u.x.iOrderByCol */
void sqlite3ExprIfTrue(Parse*, Expr*, int, int);
void sqlite3ExprIfFalse(Parse*, Expr*, int, int);
void sqlite3ExprIfFalseDup(Parse*, Expr*, int, int);
Table *sqlite3FindTable(sqlite3*,const char*, const char*);
#define LOCATE_VIEW    0x01
#define LOCATE_NOERR   0x02
Table *sqlite3LocateTable(Parse*,u32 flags,const char*, const char*);
const char *sqlite3PreferredTableName(const char*);
Table *sqlite3LocateTableItem(Parse*,u32 flags,SrcItem *);
Index *sqlite3FindIndex(sqlite3*,const char*, const char*);
void sqlite3UnlinkAndDeleteTable(sqlite3*,int,const char*);
void sqlite3UnlinkAndDeleteIndex(sqlite3*,int,const char*);
void sqlite3Vacuum(Parse*,Token*,Expr*);
int sqlite3RunVacuum(char**, sqlite3*, int, sqlite3_value*);
char *sqlite3NameFromToken(sqlite3*, const Token*);
int sqlite3ExprCompare(const Parse*,const Expr*,const Expr*, int);
int sqlite3ExprCompareSkip(Expr*,Expr*,int);
int sqlite3ExprListCompare(const ExprList*,const ExprList*, int);
int sqlite3ExprImpliesExpr(const Parse*,const Expr*,const Expr*, int);
int sqlite3ExprImpliesNonNullRow(Expr*,int);
void sqlite3AggInfoPersistWalkerInit(Walker*,Parse*);
void sqlite3ExprAnalyzeAggregates(NameContext*, Expr*);
void sqlite3ExprAnalyzeAggList(NameContext*,ExprList*);
int sqlite3ExprCoveredByIndex(Expr*, int iCur, Index *pIdx);
int sqlite3ReferencesSrcList(Parse*, Expr*, SrcList*);
Vdbe *sqlite3GetVdbe(Parse*);
#ifndef SQLITE_UNTESTABLE
void sqlite3PrngSaveState(void);
void sqlite3PrngRestoreState(void);
#endif
void sqlite3RollbackAll(sqlite3*,int);
void sqlite3CodeVerifySchema(Parse*, int);
void sqlite3CodeVerifyNamedSchema(Parse*, const char *zDb);
void sqlite3BeginTransaction(Parse*, int);
void sqlite3EndTransaction(Parse*,int);
void sqlite3Savepoint(Parse*, int, Token*);
void sqlite3CloseSavepoints(sqlite3 *);
void sqlite3LeaveMutexAndCloseZombie(sqlite3*);
u32 sqlite3IsTrueOrFalse(const char*);
int sqlite3ExprIdToTrueFalse(Expr*);
int sqlite3ExprTruthValue(const Expr*);
int sqlite3ExprIsConstant(Expr*);
int sqlite3ExprIsConstantNotJoin(Expr*);
int sqlite3ExprIsConstantOrFunction(Expr*, u8);
int sqlite3ExprIsConstantOrGroupBy(Parse*, Expr*, ExprList*);
int sqlite3ExprIsTableConstant(Expr*,int);
int sqlite3ExprIsTableConstraint(Expr*,const SrcItem*);
#ifdef SQLITE_ENABLE_CURSOR_HINTS
int sqlite3ExprContainsSubquery(Expr*);
#endif
int sqlite3ExprIsInteger(const Expr*, int*);
int sqlite3ExprCanBeNull(const Expr*);
int sqlite3ExprNeedsNoAffinityChange(const Expr*, char);
int sqlite3IsRowid(const char*);
void sqlite3GenerateRowDelete(
    Parse*,Table*,Trigger*,int,int,int,i16,u8,u8,u8,int);
void sqlite3GenerateRowIndexDelete(Parse*, Table*, int, int, int*, int);
int sqlite3GenerateIndexKey(Parse*, Index*, int, int, int, int*,Index*,int);
void sqlite3ResolvePartIdxLabel(Parse*,int);
int sqlite3ExprReferencesUpdatedColumn(Expr*,int*,int);
void sqlite3GenerateConstraintChecks(Parse*,Table*,int*,int,int,int,int,
                                     u8,u8,int,int*,int*,Upsert*);
#ifdef SQLITE_ENABLE_NULL_TRIM
  void sqlite3SetMakeRecordP5(Vdbe*,Table*);
#else
# define sqlite3SetMakeRecordP5(A,B)
#endif
void sqlite3CompleteInsertion(Parse*,Table*,int,int,int,int*,int,int,int);
int sqlite3OpenTableAndIndices(Parse*, Table*, int, u8, int, u8*, int*, int*);
void sqlite3BeginWriteOperation(Parse*, int, int);
void sqlite3MultiWrite(Parse*);
void sqlite3MayAbort(Parse*);
void sqlite3HaltConstraint(Parse*, int, int, char*, i8, u8);
void sqlite3UniqueConstraint(Parse*, int, Index*);
void sqlite3RowidConstraint(Parse*, int, Table*);
Expr *sqlite3ExprDup(sqlite3*,const Expr*,int);
ExprList *sqlite3ExprListDup(sqlite3*,const ExprList*,int);
SrcList *sqlite3SrcListDup(sqlite3*,const SrcList*,int);
Select *sqlite3SelectDup(sqlite3*,const Select*,int);
FuncDef *sqlite3FunctionSearch(int,const char*);
void sqlite3InsertBuiltinFuncs(FuncDef*,int);
FuncDef *sqlite3FindFunction(sqlite3*,const char*,int,u8,u8);
void sqlite3QuoteValue(StrAccum*,sqlite3_value*);
void sqlite3RegisterBuiltinFunctions(void);
void sqlite3RegisterDateTimeFunctions(void);
void sqlite3RegisterJsonFunctions(void);
void sqlite3RegisterPerConnectionBuiltinFunctions(sqlite3*);
#if !defined(SQLITE_OMIT_VIRTUALTABLE) && !defined(SQLITE_OMIT_JSON)
  int sqlite3JsonTableFunctions(sqlite3*);
#endif
int sqlite3SafetyCheckOk(sqlite3*);
int sqlite3SafetyCheckSickOrOk(sqlite3*);
void sqlite3ChangeCookie(Parse*, int);
With *sqlite3WithDup(sqlite3 *db, With *p);

#if !defined(SQLITE_OMIT_VIEW) && !defined(SQLITE_OMIT_TRIGGER)
void sqlite3MaterializeView(Parse*, Table*, Expr*, ExprList*,Expr*,int);
#endif

#ifndef SQLITE_OMIT_TRIGGER
  void sqlite3BeginTrigger(Parse*, Token*,Token*,int,int,IdList*,SrcList*,
                           Expr*,int, int);
  void sqlite3FinishTrigger(Parse*, TriggerStep*, Token*);
  void sqlite3DropTrigger(Parse*, SrcList*, int);
  void sqlite3DropTriggerPtr(Parse*, Trigger*);
  Trigger *sqlite3TriggersExist(Parse *, Table*, int, ExprList*, int *pMask);
  Trigger *sqlite3TriggerList(Parse *, Table *);
  void sqlite3CodeRowTrigger(Parse*, Trigger *, int, ExprList*, int, Table *,
                            int, int, int);
  void sqlite3CodeRowTriggerDirect(Parse *, Trigger *, Table *, int, int, int);
  void sqliteViewTriggers(Parse*, Table*, Expr*, int, ExprList*);
  void sqlite3DeleteTriggerStep(sqlite3*, TriggerStep*);
  TriggerStep *sqlite3TriggerSelectStep(sqlite3*,Select*,
                                        const char*,const char*);
  TriggerStep *sqlite3TriggerInsertStep(Parse*,Token*, IdList*,
                                        Select*,u8,Upsert*,
                                        const char*,const char*);
  TriggerStep *sqlite3TriggerUpdateStep(Parse*,Token*,SrcList*,ExprList*,
                                        Expr*, u8, const char*,const char*);
  TriggerStep *sqlite3TriggerDeleteStep(Parse*,Token*, Expr*,
                                        const char*,const char*);
  void sqlite3DeleteTrigger(sqlite3*, Trigger*);
  void sqlite3UnlinkAndDeleteTrigger(sqlite3*,int,const char*);
  u32 sqlite3TriggerColmask(Parse*,Trigger*,ExprList*,int,int,Table*,int);
  SrcList *sqlite3TriggerStepSrc(Parse*, TriggerStep*);
# define sqlite3ParseToplevel(p) ((p)->pToplevel ? (p)->pToplevel : (p))
# define sqlite3IsToplevel(p) ((p)->pToplevel==0)
#else
# define sqlite3TriggersExist(B,C,D,E,F) 0
# define sqlite3DeleteTrigger(A,B)
# define sqlite3DropTriggerPtr(A,B)
# define sqlite3UnlinkAndDeleteTrigger(A,B,C)
# define sqlite3CodeRowTrigger(A,B,C,D,E,F,G,H,I)
# define sqlite3CodeRowTriggerDirect(A,B,C,D,E,F)
# define sqlite3TriggerList(X, Y) 0
# define sqlite3ParseToplevel(p) p
# define sqlite3IsToplevel(p) 1
# define sqlite3TriggerColmask(A,B,C,D,E,F,G) 0
# define sqlite3TriggerStepSrc(A,B) 0
#endif

int sqlite3JoinType(Parse*, Token*, Token*, Token*);
int sqlite3ColumnIndex(Table *pTab, const char *zCol);
void sqlite3SrcItemColumnUsed(SrcItem*,int);
void sqlite3SetJoinExpr(Expr*,int,u32);
void sqlite3CreateForeignKey(Parse*, ExprList*, Token*, ExprList*, int);
void sqlite3DeferForeignKey(Parse*, int);
#ifndef SQLITE_OMIT_AUTHORIZATION
  void sqlite3AuthRead(Parse*,Expr*,Schema*,SrcList*);
  int sqlite3AuthCheck(Parse*,int, const char*, const char*, const char*);
  void sqlite3AuthContextPush(Parse*, AuthContext*, const char*);
  void sqlite3AuthContextPop(AuthContext*);
  int sqlite3AuthReadCol(Parse*, const char *, const char *, int);
#else
# define sqlite3AuthRead(a,b,c,d)
# define sqlite3AuthCheck(a,b,c,d,e)    SQLITE_OK
# define sqlite3AuthContextPush(a,b,c)
# define sqlite3AuthContextPop(a)  ((void)(a))
#endif
int sqlite3DbIsNamed(sqlite3 *db, int iDb, const char *zName);
void sqlite3Attach(Parse*, Expr*, Expr*, Expr*);
void sqlite3Detach(Parse*, Expr*);
void sqlite3FixInit(DbFixer*, Parse*, int, const char*, const Token*);
int sqlite3FixSrcList(DbFixer*, SrcList*);
int sqlite3FixSelect(DbFixer*, Select*);
int sqlite3FixExpr(DbFixer*, Expr*);
int sqlite3FixTriggerStep(DbFixer*, TriggerStep*);
int sqlite3RealSameAsInt(double,sqlite3_int64);
i64 sqlite3RealToI64(double);
int sqlite3Int64ToText(i64,char*);
int sqlite3AtoF(const char *z, double*, int, u8);
int sqlite3GetInt32(const char *, int*);
int sqlite3Atoi(const char*);
#ifndef SQLITE_OMIT_UTF16
int sqlite3Utf16ByteLen(const void *pData, int nChar);
#endif
int sqlite3Utf8CharLen(const char *pData, int nByte);
u32 sqlite3Utf8Read(const u8**);
LogEst sqlite3LogEst(u64);
LogEst sqlite3LogEstFromDouble(double);
u64 sqlite3LogEstToInt(LogEst);
VList *sqlite3VListAdd(sqlite3*,VList*,const char*,int,int);
const char *sqlite3VListNumToName(VList*,int);
int sqlite3VListNameToNum(VList*,const char*,int);

/*
** Routines to read and write variable-length integers.  These used to
** be defined locally, but now we use the varint routines in the util.c
** file.
*/
int sqlite3PutVarint(unsigned char*, u64);
u8 sqlite3GetVarint(const unsigned char *, u64 *);
u8 sqlite3GetVarint32(const unsigned char *, u32 *);
int sqlite3VarintLen(u64 v);

/*
** The common case is for a varint to be a single byte.  They following
** macros handle the common case without a procedure call, but then call
** the procedure for larger varints.
*/
#define getVarint32(A,B)  \
  (u8)((*(A)<(u8)0x80)?((B)=(u32)*(A)),1:sqlite3GetVarint32((A),(u32 *)&(B)))
#define getVarint32NR(A,B) \
  B=(u32)*(A);if(B>=0x80)sqlite3GetVarint32((A),(u32*)&(B))
#define putVarint32(A,B)  \
  (u8)(((u32)(B)<(u32)0x80)?(*(A)=(unsigned char)(B)),1:\
  sqlite3PutVarint((A),(B)))
#define getVarint    sqlite3GetVarint
#define putVarint    sqlite3PutVarint


const char *sqlite3IndexAffinityStr(sqlite3*, Index*);
char *sqlite3TableAffinityStr(sqlite3*,const Table*);
void sqlite3TableAffinity(Vdbe*, Table*, int);
char sqlite3CompareAffinity(const Expr *pExpr, char aff2);
int sqlite3IndexAffinityOk(const Expr *pExpr, char idx_affinity);
char sqlite3TableColumnAffinity(const Table*,int);
char sqlite3ExprAffinity(const Expr *pExpr);
int sqlite3ExprDataType(const Expr *pExpr);
int sqlite3Atoi64(const char*, i64*, int, u8);
int sqlite3DecOrHexToI64(const char*, i64*);
void sqlite3ErrorWithMsg(sqlite3*, int, const char*,...);
void sqlite3Error(sqlite3*,int);
void sqlite3ErrorClear(sqlite3*);
void sqlite3SystemError(sqlite3*,int);
void *sqlite3HexToBlob(sqlite3*, const char *z, int n);
int sqlite3TwoPartName(Parse *, Token *, Token *, Token **);

#if defined(SQLITE_NEED_ERR_NAME)
const char *sqlite3ErrName(int);
#endif

#ifndef SQLITE_OMIT_DESERIALIZE
int sqlite3MemdbInit(void);
int sqlite3IsMemdb(const sqlite3_vfs*);
#else
# define sqlite3IsMemdb(X) 0
#endif

const char *sqlite3ErrStr(int);
int sqlite3ReadSchema(Parse *pParse);
CollSeq *sqlite3FindCollSeq(sqlite3*,u8 enc, const char*,int);
int sqlite3IsBinary(const CollSeq*);
CollSeq *sqlite3LocateCollSeq(Parse *pParse, const char*zName);
void sqlite3SetTextEncoding(sqlite3 *db, u8);
CollSeq *sqlite3ExprCollSeq(Parse *pParse, const Expr *pExpr);
CollSeq *sqlite3ExprNNCollSeq(Parse *pParse, const Expr *pExpr);
int sqlite3ExprCollSeqMatch(Parse*,const Expr*,const Expr*);
Expr *sqlite3ExprAddCollateToken(const Parse *pParse, Expr*, const Token*, int);
Expr *sqlite3ExprAddCollateString(const Parse*,Expr*,const char*);
Expr *sqlite3ExprSkipCollate(Expr*);
Expr *sqlite3ExprSkipCollateAndLikely(Expr*);
int sqlite3CheckCollSeq(Parse *, CollSeq *);
int sqlite3WritableSchema(sqlite3*);
int sqlite3CheckObjectName(Parse*, const char*,const char*,const char*);
void sqlite3VdbeSetChanges(sqlite3 *, i64);
#ifdef SQLITE_ENABLE_8_3_NAMES
void sqlite3FileSuffix3(const char*, char*);
#else
# define sqlite3FileSuffix3(X,Y)
#endif
u8 sqlite3GetBoolean(const char *z,u8);

const void *sqlite3ValueText(sqlite3_value*, u8);
int sqlite3ValueBytes(sqlite3_value*, u8);
void sqlite3ValueSetStr(sqlite3_value*, int, const void *,u8,
                        void(*)(void*));
void sqlite3ValueSetNull(sqlite3_value*);
void sqlite3ValueFree(sqlite3_value*);
#ifndef SQLITE_UNTESTABLE
void sqlite3ResultIntReal(sqlite3_context*);
#endif
sqlite3_value *sqlite3ValueNew(sqlite3 *);
#ifndef SQLITE_OMIT_UTF16
char *sqlite3Utf16to8(sqlite3 *, const void*, int, u8);
#endif
int sqlite3ValueFromExpr(sqlite3 *, const Expr *, u8, u8, sqlite3_value **);
void sqlite3ValueApplyAffinity(sqlite3_value *, u8, u8);
#ifndef SQLITE_AMALGAMATION
extern const unsigned char sqlite3OpcodeProperty[];
extern const char sqlite3StrBINARY[];
extern const unsigned char sqlite3StdTypeLen[];
extern const char sqlite3StdTypeAffinity[];
extern const char *sqlite3StdType[];
extern const unsigned char sqlite3UpperToLower[];
extern const unsigned char *sqlite3aLTb;
extern const unsigned char *sqlite3aEQb;
extern const unsigned char *sqlite3aGTb;
extern const unsigned char sqlite3CtypeMap[];
extern SQLITE_WSD struct Sqlite3Config sqlite3Config;
extern FuncDefHash sqlite3BuiltinFunctions;
#ifndef SQLITE_OMIT_WSD
extern int sqlite3PendingByte;
#endif
#endif /* SQLITE_AMALGAMATION */
#ifdef VDBE_PROFILE
extern sqlite3_uint64 sqlite3NProfileCnt;
#endif
void sqlite3RootPageMoved(sqlite3*, int, Pgno, Pgno);
void sqlite3Reindex(Parse*, Token*, Token*);
void sqlite3AlterFunctions(void);
void sqlite3AlterRenameTable(Parse*, SrcList*, Token*);
void sqlite3AlterRenameColumn(Parse*, SrcList*, Token*, Token*);
int sqlite3GetToken(const unsigned char *, int *);
void sqlite3NestedParse(Parse*, const char*, ...);
void sqlite3ExpirePreparedStatements(sqlite3*, int);
void sqlite3CodeRhsOfIN(Parse*, Expr*, int);
int sqlite3CodeSubselect(Parse*, Expr*);
void sqlite3SelectPrep(Parse*, Select*, NameContext*);
int sqlite3ExpandSubquery(Parse*, SrcItem*);
void sqlite3SelectWrongNumTermsError(Parse *pParse, Select *p);
int sqlite3MatchEName(
  const struct ExprList_item*,
  const char*,
  const char*,
  const char*
);
Bitmask sqlite3ExprColUsed(Expr*);
int sqlite3ResolveExprNames(NameContext*, Expr*);
int sqlite3ResolveExprListNames(NameContext*, ExprList*);
void sqlite3ResolveSelectNames(Parse*, Select*, NameContext*);
int sqlite3ResolveSelfReference(Parse*,Table*,int,Expr*,ExprList*);
int sqlite3ResolveOrderGroupBy(Parse*, Select*, ExprList*, const char*);
void sqlite3ColumnDefault(Vdbe *, Table *, int, int);
void sqlite3AlterFinishAddColumn(Parse *, Token *);
void sqlite3AlterBeginAddColumn(Parse *, SrcList *);
void sqlite3AlterDropColumn(Parse*, SrcList*, const Token*);
const void *sqlite3RenameTokenMap(Parse*, const void*, const Token*);
void sqlite3RenameTokenRemap(Parse*, const void *pTo, const void *pFrom);
void sqlite3RenameExprUnmap(Parse*, Expr*);
void sqlite3RenameExprlistUnmap(Parse*, ExprList*);
CollSeq *sqlite3GetCollSeq(Parse*, u8, CollSeq *, const char*);
char sqlite3AffinityType(const char*, Column*);
void sqlite3Analyze(Parse*, Token*, Token*);
int sqlite3InvokeBusyHandler(BusyHandler*);
int sqlite3FindDb(sqlite3*, Token*);
int sqlite3FindDbName(sqlite3 *, const char *);
int sqlite3AnalysisLoad(sqlite3*,int iDB);
void sqlite3DeleteIndexSamples(sqlite3*,Index*);
void sqlite3DefaultRowEst(Index*);
void sqlite3RegisterLikeFunctions(sqlite3*, int);
int sqlite3IsLikeFunction(sqlite3*,Expr*,int*,char*);
void sqlite3SchemaClear(void *);
Schema *sqlite3SchemaGet(sqlite3 *, Btree *);
int sqlite3SchemaToIndex(sqlite3 *db, Schema *);
KeyInfo *sqlite3KeyInfoAlloc(sqlite3*,int,int);
void sqlite3KeyInfoUnref(KeyInfo*);
KeyInfo *sqlite3KeyInfoRef(KeyInfo*);
KeyInfo *sqlite3KeyInfoOfIndex(Parse*, Index*);
KeyInfo *sqlite3KeyInfoFromExprList(Parse*, ExprList*, int, int);
const char *sqlite3SelectOpName(int);
int sqlite3HasExplicitNulls(Parse*, ExprList*);

#ifdef SQLITE_DEBUG
int sqlite3KeyInfoIsWriteable(KeyInfo*);
#endif
int sqlite3CreateFunc(sqlite3 *, const char *, int, int, void *,
  void (*)(sqlite3_context*,int,sqlite3_value **),
  void (*)(sqlite3_context*,int,sqlite3_value **), 
  void (*)(sqlite3_context*),
  void (*)(sqlite3_context*),
  void (*)(sqlite3_context*,int,sqlite3_value **), 
  FuncDestructor *pDestructor
);
void sqlite3NoopDestructor(void*);
void *sqlite3OomFault(sqlite3*);
void sqlite3OomClear(sqlite3*);
int sqlite3ApiExit(sqlite3 *db, int);
int sqlite3OpenTempDatabase(Parse *);

void sqlite3StrAccumInit(StrAccum*, sqlite3*, char*, int, int);
int sqlite3StrAccumEnlarge(StrAccum*, i64);
char *sqlite3StrAccumFinish(StrAccum*);
void sqlite3StrAccumSetError(StrAccum*, u8);
void sqlite3ResultStrAccum(sqlite3_context*,StrAccum*);
void sqlite3SelectDestInit(SelectDest*,int,int);
Expr *sqlite3CreateColumnExpr(sqlite3 *, SrcList *, int, int);
void sqlite3RecordErrorByteOffset(sqlite3*,const char*);
void sqlite3RecordErrorOffsetOfExpr(sqlite3*,const Expr*);

void sqlite3BackupRestart(sqlite3_backup *);
void sqlite3BackupUpdate(sqlite3_backup *, Pgno, const u8 *);

#ifndef SQLITE_OMIT_SUBQUERY
int sqlite3ExprCheckIN(Parse*, Expr*);
#else
# define sqlite3ExprCheckIN(x,y) SQLITE_OK
#endif

#ifdef SQLITE_ENABLE_STAT4
int sqlite3Stat4ProbeSetValue(
    Parse*,Index*,UnpackedRecord**,Expr*,int,int,int*);
int sqlite3Stat4ValueFromExpr(Parse*, Expr*, u8, sqlite3_value**);
void sqlite3Stat4ProbeFree(UnpackedRecord*);
int sqlite3Stat4Column(sqlite3*, const void*, int, int, sqlite3_value**);
char sqlite3IndexColumnAffinity(sqlite3*, Index*, int);
#endif

/*
** The interface to the LEMON-generated parser
*/
#ifndef SQLITE_AMALGAMATION
  void *sqlite3ParserAlloc(void*(*)(u64), Parse*);
  void sqlite3ParserFree(void*, void(*)(void*));
#endif
void sqlite3Parser(void*, int, Token);
int sqlite3ParserFallback(int);
#ifdef YYTRACKMAXSTACKDEPTH
  int sqlite3ParserStackPeak(void*);
#endif

void sqlite3AutoLoadExtensions(sqlite3*);
#ifndef SQLITE_OMIT_LOAD_EXTENSION
  void sqlite3CloseExtensions(sqlite3*);
#else
# define sqlite3CloseExtensions(X)
#endif

#ifndef SQLITE_OMIT_SHARED_CACHE
  void sqlite3TableLock(Parse *, int, Pgno, u8, const char *);
#else
  #define sqlite3TableLock(v,w,x,y,z)
#endif

#ifdef SQLITE_TEST
  int sqlite3Utf8To8(unsigned char*);
#endif

#ifdef SQLITE_OMIT_VIRTUALTABLE
#  define sqlite3VtabClear(D,T)
#  define sqlite3VtabSync(X,Y) SQLITE_OK
#  define sqlite3VtabRollback(X)
#  define sqlite3VtabCommit(X)
#  define sqlite3VtabInSync(db) 0
#  define sqlite3VtabLock(X)
#  define sqlite3VtabUnlock(X)
#  define sqlite3VtabModuleUnref(D,X)
#  define sqlite3VtabUnlockList(X)
#  define sqlite3VtabSavepoint(X, Y, Z) SQLITE_OK
#  define sqlite3GetVTable(X,Y)  ((VTable*)0)
#else
   void sqlite3VtabClear(sqlite3 *db, Table*);
   void sqlite3VtabDisconnect(sqlite3 *db, Table *p);
   int sqlite3VtabSync(sqlite3 *db, Vdbe*);
   int sqlite3VtabRollback(sqlite3 *db);
   int sqlite3VtabCommit(sqlite3 *db);
   void sqlite3VtabLock(VTable *);
   void sqlite3VtabUnlock(VTable *);
   void sqlite3VtabModuleUnref(sqlite3*,Module*);
   void sqlite3VtabUnlockList(sqlite3*);
   int sqlite3VtabSavepoint(sqlite3 *, int, int);
   void sqlite3VtabImportErrmsg(Vdbe*, sqlite3_vtab*);
   VTable *sqlite3GetVTable(sqlite3*, Table*);
   Module *sqlite3VtabCreateModule(
     sqlite3*,
     const char*,
     const sqlite3_module*,
     void*,
     void(*)(void*)
   );
#  define sqlite3VtabInSync(db) ((db)->nVTrans>0 && (db)->aVTrans==0)
#endif
int sqlite3ReadOnlyShadowTables(sqlite3 *db);
#ifndef SQLITE_OMIT_VIRTUALTABLE
  int sqlite3ShadowTableName(sqlite3 *db, const char *zName);
  int sqlite3IsShadowTableOf(sqlite3*,Table*,const char*);
  void sqlite3MarkAllShadowTablesOf(sqlite3*, Table*);
#else
# define sqlite3ShadowTableName(A,B) 0
# define sqlite3IsShadowTableOf(A,B,C) 0
# define sqlite3MarkAllShadowTablesOf(A,B)
#endif
int sqlite3VtabEponymousTableInit(Parse*,Module*);
void sqlite3VtabEponymousTableClear(sqlite3*,Module*);
void sqlite3VtabMakeWritable(Parse*,Table*);
void sqlite3VtabBeginParse(Parse*, Token*, Token*, Token*, int);
void sqlite3VtabFinishParse(Parse*, Token*);
void sqlite3VtabArgInit(Parse*);
void sqlite3VtabArgExtend(Parse*, Token*);
int sqlite3VtabCallCreate(sqlite3*, int, const char *, char **);
int sqlite3VtabCallConnect(Parse*, Table*);
int sqlite3VtabCallDestroy(sqlite3*, int, const char *);
int sqlite3VtabBegin(sqlite3 *, VTable *);

FuncDef *sqlite3VtabOverloadFunction(sqlite3 *,FuncDef*, int nArg, Expr*);
#if (defined(SQLITE_ENABLE_DBPAGE_VTAB) || defined(SQLITE_TEST)) \
    && !defined(SQLITE_OMIT_VIRTUALTABLE)
  void sqlite3VtabUsesAllSchemas(sqlite3_index_info*);
#endif
sqlite3_int64 sqlite3StmtCurrentTime(sqlite3_context*);
int sqlite3VdbeParameterIndex(Vdbe*, const char*, int);
int sqlite3TransferBindings(sqlite3_stmt *, sqlite3_stmt *);
void sqlite3ParseObjectInit(Parse*,sqlite3*);
void sqlite3ParseObjectReset(Parse*);
void *sqlite3ParserAddCleanup(Parse*,void(*)(sqlite3*,void*),void*);
#ifdef SQLITE_ENABLE_NORMALIZE
char *sqlite3Normalize(Vdbe*, const char*);
#endif
int sqlite3Reprepare(Vdbe*);
void sqlite3ExprListCheckLength(Parse*, ExprList*, const char*);
CollSeq *sqlite3ExprCompareCollSeq(Parse*,const Expr*);
CollSeq *sqlite3BinaryCompareCollSeq(Parse *, const Expr*, const Expr*);
int sqlite3TempInMemory(const sqlite3*);
const char *sqlite3JournalModename(int);
#ifndef SQLITE_OMIT_WAL
  int sqlite3Checkpoint(sqlite3*, int, int, int*, int*);
  int sqlite3WalDefaultHook(void*,sqlite3*,const char*,int);
#endif
#ifndef SQLITE_OMIT_CTE
  Cte *sqlite3CteNew(Parse*,Token*,ExprList*,Select*,u8);
  void sqlite3CteDelete(sqlite3*,Cte*);
  With *sqlite3WithAdd(Parse*,With*,Cte*);
  void sqlite3WithDelete(sqlite3*,With*);
  With *sqlite3WithPush(Parse*, With*, u8);
#else
# define sqlite3CteNew(P,T,E,S)   ((void*)0)
# define sqlite3CteDelete(D,C)
# define sqlite3CteWithAdd(P,W,C) ((void*)0)
# define sqlite3WithDelete(x,y)
# define sqlite3WithPush(x,y,z) ((void*)0)
#endif
#ifndef SQLITE_OMIT_UPSERT
  Upsert *sqlite3UpsertNew(sqlite3*,ExprList*,Expr*,ExprList*,Expr*,Upsert*);
  void sqlite3UpsertDelete(sqlite3*,Upsert*);
  Upsert *sqlite3UpsertDup(sqlite3*,Upsert*);
  int sqlite3UpsertAnalyzeTarget(Parse*,SrcList*,Upsert*);
  void sqlite3UpsertDoUpdate(Parse*,Upsert*,Table*,Index*,int);
  Upsert *sqlite3UpsertOfIndex(Upsert*,Index*);
  int sqlite3UpsertNextIsIPK(Upsert*);
#else
#define sqlite3UpsertNew(u,v,w,x,y,z) ((Upsert*)0)
#define sqlite3UpsertDelete(x,y)
#define sqlite3UpsertDup(x,y)         ((Upsert*)0)
#define sqlite3UpsertOfIndex(x,y)     ((Upsert*)0)
#define sqlite3UpsertNextIsIPK(x)     0
#endif


/* Declarations for functions in fkey.c. All of these are replaced by
** no-op macros if OMIT_FOREIGN_KEY is defined. In this case no foreign
** key functionality is available. If OMIT_TRIGGER is defined but
** OMIT_FOREIGN_KEY is not, only some of the functions are no-oped. In
** this case foreign keys are parsed, but no other functionality is
** provided (enforcement of FK constraints requires the triggers sub-system).
*/
#if !defined(SQLITE_OMIT_FOREIGN_KEY) && !defined(SQLITE_OMIT_TRIGGER)
  void sqlite3FkCheck(Parse*, Table*, int, int, int*, int);
  void sqlite3FkDropTable(Parse*, SrcList *, Table*);
  void sqlite3FkActions(Parse*, Table*, ExprList*, int, int*, int);
  int sqlite3FkRequired(Parse*, Table*, int*, int);
  u32 sqlite3FkOldmask(Parse*, Table*);
  FKey *sqlite3FkReferences(Table *);
  void sqlite3FkClearTriggerCache(sqlite3*,int);
#else
  #define sqlite3FkActions(a,b,c,d,e,f)
  #define sqlite3FkCheck(a,b,c,d,e,f)
  #define sqlite3FkDropTable(a,b,c)
  #define sqlite3FkOldmask(a,b)         0
  #define sqlite3FkRequired(a,b,c,d)    0
  #define sqlite3FkReferences(a)        0
  #define sqlite3FkClearTriggerCache(a,b)
#endif
#ifndef SQLITE_OMIT_FOREIGN_KEY
  void sqlite3FkDelete(sqlite3 *, Table*);
  int sqlite3FkLocateIndex(Parse*,Table*,FKey*,Index**,int**);
#else
  #define sqlite3FkDelete(a,b)
  #define sqlite3FkLocateIndex(a,b,c,d,e)
#endif


/*
** Available fault injectors.  Should be numbered beginning with 0.
*/
#define SQLITE_FAULTINJECTOR_MALLOC     0
#define SQLITE_FAULTINJECTOR_COUNT      1

/*
** The interface to the code in fault.c used for identifying "benign"
** malloc failures. This is only present if SQLITE_UNTESTABLE
** is not defined.
*/
#ifndef SQLITE_UNTESTABLE
  void sqlite3BeginBenignMalloc(void);
  void sqlite3EndBenignMalloc(void);
#else
  #define sqlite3BeginBenignMalloc()
  #define sqlite3EndBenignMalloc()
#endif

/*
** Allowed return values from sqlite3FindInIndex()
*/
#define IN_INDEX_ROWID        1   /* Search the rowid of the table */
#define IN_INDEX_EPH          2   /* Search an ephemeral b-tree */
#define IN_INDEX_INDEX_ASC    3   /* Existing index ASCENDING */
#define IN_INDEX_INDEX_DESC   4   /* Existing index DESCENDING */
#define IN_INDEX_NOOP         5   /* No table available. Use comparisons */
/*
** Allowed flags for the 3rd parameter to sqlite3FindInIndex().
*/
#define IN_INDEX_NOOP_OK     0x0001  /* OK to return IN_INDEX_NOOP */
#define IN_INDEX_MEMBERSHIP  0x0002  /* IN operator used for membership test */
#define IN_INDEX_LOOP        0x0004  /* IN operator used as a loop */
int sqlite3FindInIndex(Parse *, Expr *, u32, int*, int*, int*);

int sqlite3JournalOpen(sqlite3_vfs *, const char *, sqlite3_file *, int, int);
int sqlite3JournalSize(sqlite3_vfs *);
#if defined(SQLITE_ENABLE_ATOMIC_WRITE) \
 || defined(SQLITE_ENABLE_BATCH_ATOMIC_WRITE)
  int sqlite3JournalCreate(sqlite3_file *);
#endif

int sqlite3JournalIsInMemory(sqlite3_file *p);
void sqlite3MemJournalOpen(sqlite3_file *);

void sqlite3ExprSetHeightAndFlags(Parse *pParse, Expr *p);
#if SQLITE_MAX_EXPR_DEPTH>0
  int sqlite3SelectExprHeight(const Select *);
  int sqlite3ExprCheckHeight(Parse*, int);
#else
  #define sqlite3SelectExprHeight(x) 0
  #define sqlite3ExprCheckHeight(x,y)
#endif

u32 sqlite3Get4byte(const u8*);
void sqlite3Put4byte(u8*, u32);

#ifdef SQLITE_ENABLE_UNLOCK_NOTIFY
  void sqlite3ConnectionBlocked(sqlite3 *, sqlite3 *);
  void sqlite3ConnectionUnlocked(sqlite3 *db);
  void sqlite3ConnectionClosed(sqlite3 *db);
#else
  #define sqlite3ConnectionBlocked(x,y)
  #define sqlite3ConnectionUnlocked(x)
  #define sqlite3ConnectionClosed(x)
#endif

#ifdef SQLITE_DEBUG
  void sqlite3ParserTrace(FILE*, char *);
#endif
#if defined(YYCOVERAGE)
  int sqlite3ParserCoverage(FILE*);
#endif

/*
** If the SQLITE_ENABLE IOTRACE exists then the global variable
** sqlite3IoTrace is a pointer to a printf-like routine used to
** print I/O tracing messages.
*/
#ifdef SQLITE_ENABLE_IOTRACE
# define IOTRACE(A)  if( sqlite3IoTrace ){ sqlite3IoTrace A; }
  void sqlite3VdbeIOTraceSql(Vdbe*);
SQLITE_API SQLITE_EXTERN void (SQLITE_CDECL *sqlite3IoTrace)(const char*,...);
#else
# define IOTRACE(A)
# define sqlite3VdbeIOTraceSql(X)
#endif

/*
** Threading interface
*/
#if SQLITE_MAX_WORKER_THREADS>0
int sqlite3ThreadCreate(SQLiteThread**,void*(*)(void*),void*);
int sqlite3ThreadJoin(SQLiteThread*, void**);
#endif

#if defined(SQLITE_ENABLE_DBPAGE_VTAB) || defined(SQLITE_TEST)
int sqlite3DbpageRegister(sqlite3*);
#endif
#if defined(SQLITE_ENABLE_DBSTAT_VTAB) || defined(SQLITE_TEST)
int sqlite3DbstatRegister(sqlite3*);
#endif

Expr *sqlite3VectorFieldSubexpr(Expr*, int);
Expr *sqlite3ExprForVectorField(Parse*,Expr*,int,int);
void sqlite3VectorErrorMsg(Parse*, Expr*);

#ifndef SQLITE_OMIT_COMPILEOPTION_DIAGS
const char **sqlite3CompileOptions(int *pnOpt);
#endif

#if SQLITE_OS_UNIX && defined(SQLITE_OS_KV_OPTIONAL)
int sqlite3KvvfsInit(void);
#endif

#if defined(VDBE_PROFILE) \
 || defined(SQLITE_PERFORMANCE_TRACE) \
 || defined(SQLITE_ENABLE_STMT_SCANSTATUS)
sqlite3_uint64 sqlite3Hwtime(void);
#endif

#ifdef SQLITE_ENABLE_STMT_SCANSTATUS
# define IS_STMT_SCANSTATUS(db) (db->flags & SQLITE_StmtScanStatus)
#else
# define IS_STMT_SCANSTATUS(db) 0
#endif

#endif /* SQLITEINT_H */
