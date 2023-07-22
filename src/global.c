/*
** 2008 June 13
**
** The author disclaims copyright to this source code.  In place of
** a legal notice, here is a blessing:
**
**    May you do good and not evil.
**    May you find forgiveness for yourself and forgive others.
**    May you share freely, never taking more than you give.
**
*************************************************************************
**
** This file contains definitions of global variables and constants.
*/
#include "sqliteInt.h"
#include "sqlite3_rs.h"

const unsigned char *sqlite3aLTb = &sqlite3UpperToLower[256-OP_Ne];
const unsigned char *sqlite3aEQb = &sqlite3UpperToLower[256+6-OP_Ne];
const unsigned char *sqlite3aGTb = &sqlite3UpperToLower[256+12-OP_Ne];

/*
** The following 256 byte lookup table is used to support SQLites built-in
** equivalents to the following standard library functions:
**
**   isspace()                        0x01
**   isalpha()                        0x02
**   isdigit()                        0x04
**   isalnum()                        0x06
**   isxdigit()                       0x08
**   toupper()                        0x20
**   SQLite identifier character      0x40
**   Quote character                  0x80
**
** Bit 0x20 is set if the mapped character requires translation to upper
** case. i.e. if the character is a lower-case ASCII character.
** If x is a lower-case ASCII character, then its upper-case equivalent
** is (x - 0x20). Therefore toupper() can be implemented as:
**
**   (x & ~(map[x]&0x20))
**
** The equivalent of tolower() is implemented using the sqlite3UpperToLower[]
** array. tolower() is used more often than toupper() by SQLite.
**
** Bit 0x40 is set if the character is non-alphanumeric and can be used in an 
** SQLite identifier.  Identifiers are alphanumerics, "_", "$", and any
** non-ASCII UTF character. Hence the test for whether or not a character is
** part of an identifier is 0x46.
*/
const unsigned char sqlite3CtypeMap[256] = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  /* 00..07    ........ */
  0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00,  /* 08..0f    ........ */
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  /* 10..17    ........ */
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  /* 18..1f    ........ */
  0x01, 0x00, 0x80, 0x00, 0x40, 0x00, 0x00, 0x80,  /* 20..27     !"#$%&' */
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  /* 28..2f    ()*+,-./ */
  0x0c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c,  /* 30..37    01234567 */
  0x0c, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  /* 38..3f    89:;<=>? */

  0x00, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x02,  /* 40..47    @ABCDEFG */
  0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,  /* 48..4f    HIJKLMNO */
  0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,  /* 50..57    PQRSTUVW */
  0x02, 0x02, 0x02, 0x80, 0x00, 0x00, 0x00, 0x40,  /* 58..5f    XYZ[\]^_ */
  0x80, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x2a, 0x22,  /* 60..67    `abcdefg */
  0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,  /* 68..6f    hijklmno */
  0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,  /* 70..77    pqrstuvw */
  0x22, 0x22, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00,  /* 78..7f    xyz{|}~. */

  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* 80..87    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* 88..8f    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* 90..97    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* 98..9f    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* a0..a7    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* a8..af    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* b0..b7    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* b8..bf    ........ */

  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* c0..c7    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* c8..cf    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* d0..d7    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* d8..df    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* e0..e7    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* e8..ef    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,  /* f0..f7    ........ */
  0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40   /* f8..ff    ........ */
};

/* EVIDENCE-OF: R-02982-34736 In order to maintain full backwards
** compatibility for legacy applications, the URI filename capability is
** disabled by default.
**
** EVIDENCE-OF: R-38799-08373 URI filenames can be enabled or disabled
** using the SQLITE_USE_URI=1 or SQLITE_USE_URI=0 compile-time options.
**
** EVIDENCE-OF: R-43642-56306 By default, URI handling is globally
** disabled. The default value may be changed by compiling with the
** SQLITE_USE_URI symbol defined.
*/
#ifndef SQLITE_USE_URI
# define SQLITE_USE_URI 0
#endif

/* EVIDENCE-OF: R-38720-18127 The default setting is determined by the
** SQLITE_ALLOW_COVERING_INDEX_SCAN compile-time option, or is "on" if
** that compile-time option is omitted.
*/
#if !defined(SQLITE_ALLOW_COVERING_INDEX_SCAN)
# define SQLITE_ALLOW_COVERING_INDEX_SCAN 1
#else
# if !SQLITE_ALLOW_COVERING_INDEX_SCAN 
#   error "Compile-time disabling of covering index scan using the\
 -DSQLITE_ALLOW_COVERING_INDEX_SCAN=0 option is deprecated.\
 Contact SQLite developers if this is a problem for you, and\
 delete this #error macro to continue with your build."
# endif
#endif

/* The minimum PMA size is set to this value multiplied by the database
** page size in bytes.
*/
#ifndef SQLITE_SORTER_PMASZ
# define SQLITE_SORTER_PMASZ 250
#endif

/* Statement journals spill to disk when their size exceeds the following
** threshold (in bytes). 0 means that statement journals are created and
** written to disk immediately (the default behavior for SQLite versions
** before 3.12.0).  -1 means always keep the entire statement journal in
** memory.  (The statement journal is also always held entirely in memory
** if journal_mode=MEMORY or if temp_store=MEMORY, regardless of this
** setting.)
*/
#ifndef SQLITE_STMTJRNL_SPILL 
# define SQLITE_STMTJRNL_SPILL (64*1024)
#endif

/*
** The default lookaside-configuration, the format "SZ,N".  SZ is the
** number of bytes in each lookaside slot (should be a multiple of 8)
** and N is the number of slots.  The lookaside-configuration can be
** changed as start-time using sqlite3_config(SQLITE_CONFIG_LOOKASIDE)
** or at run-time for an individual database connection using
** sqlite3_db_config(db, SQLITE_DBCONFIG_LOOKASIDE);
**
** With the two-size-lookaside enhancement, less lookaside is required.
** The default configuration of 1200,40 actually provides 30 1200-byte slots
** and 93 128-byte slots, which is more lookaside than is available
** using the older 1200,100 configuration without two-size-lookaside.
*/
#ifndef SQLITE_DEFAULT_LOOKASIDE
# ifdef SQLITE_OMIT_TWOSIZE_LOOKASIDE
#   define SQLITE_DEFAULT_LOOKASIDE 1200,100  /* 120KB of memory */
# else
#   define SQLITE_DEFAULT_LOOKASIDE 1200,40   /* 48KB of memory */
# endif
#endif


/* The default maximum size of an in-memory database created using
** sqlite3_deserialize()
*/
#ifndef SQLITE_MEMDB_DEFAULT_MAXSIZE
# define SQLITE_MEMDB_DEFAULT_MAXSIZE 1073741824
#endif

/*
** The following singleton contains the global configuration for
** the SQLite library.
*/
SQLITE_WSD struct Sqlite3Config sqlite3Config = {
   SQLITE_DEFAULT_MEMSTATUS,  /* bMemstat */
   1,                         /* bCoreMutex */
   SQLITE_THREADSAFE==1,      /* bFullMutex */
   SQLITE_USE_URI,            /* bOpenUri */
   SQLITE_ALLOW_COVERING_INDEX_SCAN,   /* bUseCis */
   0,                         /* bSmallMalloc */
   1,                         /* bExtraSchemaChecks */
   0x7ffffffe,                /* mxStrlen */
   0,                         /* neverCorrupt */
   SQLITE_DEFAULT_LOOKASIDE,  /* szLookaside, nLookaside */
   SQLITE_STMTJRNL_SPILL,     /* nStmtSpill */
   {0,0,0,0,0,0,0,0},         /* m */
   {0,0,0,0,0,0,0,0,0},       /* mutex */
   {0,0,0,0,0,0,0,0,0,0,0,0,0},/* pcache2 */
   (void*)0,                  /* pHeap */
   0,                         /* nHeap */
   0, 0,                      /* mnHeap, mxHeap */
   SQLITE_DEFAULT_MMAP_SIZE,  /* szMmap */
   SQLITE_MAX_MMAP_SIZE,      /* mxMmap */
   (void*)0,                  /* pPage */
   0,                         /* szPage */
   SQLITE_DEFAULT_PCACHE_INITSZ, /* nPage */
   0,                         /* mxParserStack */
   0,                         /* sharedCacheEnabled */
   SQLITE_SORTER_PMASZ,       /* szPma */
   /* All the rest should always be initialized to zero */
   0,                         /* isInit */
   0,                         /* inProgress */
   0,                         /* isMutexInit */
   0,                         /* isMallocInit */
   0,                         /* isPCacheInit */
   0,                         /* nRefInitMutex */
   0,                         /* pInitMutex */
   0,                         /* xLog */
   0,                         /* pLogArg */
#ifdef SQLITE_ENABLE_SQLLOG
   0,                         /* xSqllog */
   0,                         /* pSqllogArg */
#endif
#ifdef SQLITE_VDBE_COVERAGE
   0,                         /* xVdbeBranch */
   0,                         /* pVbeBranchArg */
#endif
#ifndef SQLITE_OMIT_DESERIALIZE
   SQLITE_MEMDB_DEFAULT_MAXSIZE,   /* mxMemdbSize */
#endif
#ifndef SQLITE_UNTESTABLE
   0,                         /* xTestCallback */
#endif
   0,                         /* bLocaltimeFault */
   0,                         /* xAltLocaltime */
   0x7ffffffe,                /* iOnceResetThreshold */
   SQLITE_DEFAULT_SORTERREF_SIZE,   /* szSorterRef */
   0,                         /* iPrngSeed */
#ifdef SQLITE_DEBUG
   {0,0,0,0,0,0},             /* aTune */
#endif
};

/*
** Hash table for global functions - functions common to all
** database connections.  After initialization, this table is
** read-only.
*/
FuncDefHash sqlite3BuiltinFunctions;

#if defined(SQLITE_COVERAGE_TEST) || defined(SQLITE_DEBUG)
/*
** Counter used for coverage testing.  Does not come into play for
** release builds.
**
** Access to this global variable is not mutex protected.  This might
** result in TSAN warnings.  But as the variable does not exist in
** release builds, that should not be a concern.
*/
unsigned int sqlite3CoverageCounter;
#endif /* SQLITE_COVERAGE_TEST || SQLITE_DEBUG */

#ifdef VDBE_PROFILE
/*
** The following performance counter can be used in place of
** sqlite3Hwtime() for profiling.  This is a no-op on standard builds.
*/
sqlite3_uint64 sqlite3NProfileCnt = 0;
#endif

/*
** The value of the "pending" byte must be 0x40000000 (1 byte past the
** 1-gibabyte boundary) in a compatible database.  SQLite never uses
** the database page that contains the pending byte.  It never attempts
** to read or write that page.  The pending byte page is set aside
** for use by the VFS layers as space for managing file locks.
**
** During testing, it is often desirable to move the pending byte to
** a different position in the file.  This allows code that has to
** deal with the pending byte to run on files that are much smaller
** than 1 GiB.  The sqlite3_test_control() interface can be used to
** move the pending byte.
**
** IMPORTANT:  Changing the pending byte to any value other than
** 0x40000000 results in an incompatible database file format!
** Changing the pending byte during operation will result in undefined
** and incorrect behavior.
*/
#ifndef SQLITE_OMIT_WSD
int sqlite3PendingByte = 0x40000000;
#endif

/*
** Tracing flags set by SQLITE_TESTCTRL_TRACEFLAGS.
*/
u32 sqlite3TreeTrace = 0;
u32 sqlite3WhereTrace = 0;

#include "opcodes.h"
/*
** Properties of opcodes.  The OPFLG_INITIALIZER macro is
** created by mkopcodeh.awk during compilation.  Data is obtained
** from the comments following the "case OP_xxxx:" statements in
** the vdbe.c file.  
*/
const unsigned char sqlite3OpcodeProperty[] = OPFLG_INITIALIZER;

/*
** Name of the default collating sequence
*/
const char sqlite3StrBINARY[] = "BINARY";

// TODO: remove this once there are no more uses in C
const unsigned char sqlite3StdTypeLen[] = { 3, 4, 3, 7, 4, 4 };
const char sqlite3StdTypeAffinity[] = {
  SQLITE_AFF_NUMERIC,
  SQLITE_AFF_BLOB,
  SQLITE_AFF_INTEGER,
  SQLITE_AFF_INTEGER,
  SQLITE_AFF_REAL,
  SQLITE_AFF_TEXT
};
const char *sqlite3StdType[] = {
  "ANY",
  "BLOB",
  "INT",
  "INTEGER",
  "REAL",
  "TEXT"
};
